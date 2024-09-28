use kzg::{common_utils::{reverse_bit_order, reverse_bits_limited}, eip_4844::{blob_to_polynomial, CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_BLOB, FIELD_ELEMENTS_PER_CELL, FIELD_ELEMENTS_PER_EXT_BLOB}, Fr, G1Mul, KZGSettings, G1};
use crate::{fft_fr::fft_fr_fast, fft_g1::fft_g1_fast, fk20_proofs, kzg_proofs::g1_linear_combination, types::g1::FsG1};

use crate::types::{fft_settings::FsFFTSettings, fr::FsFr, kzg_settings::FsKZGSettings, poly::FsPoly};

fn fr_ifft(output: &mut [FsFr], input: &[FsFr], s: &FsFFTSettings) -> Result<(), String> {
    let stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();

    fft_fr_fast(output, &input, 1, &s.reverse_roots_of_unity, stride);

    let inv_len = FsFr::from_u64(input.len().try_into().unwrap()).inverse();
    for el in output {
        *el = el.mul(&inv_len);
    }

    Ok(())
}

fn fr_fft(output: &mut [FsFr], input: &[FsFr], s: &FsFFTSettings) -> Result<(), String> {
    let roots_stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();
    fft_fr_fast(output, &input, 1, &s.roots_of_unity, roots_stride);

    Ok(())
}

fn poly_lagrange_to_monomial(output: &mut [FsFr], mut largrange_poly: &[FsFr], s: &FsFFTSettings) -> Result<(), String> {
    reverse_bit_order(&mut largrange_poly.coeffs)?;

    fr_ifft(output, &largrange_poly.coeffs, s)?;

    Ok(())
}

fn toeplitz_coeffs_stride(out: &mut [FsFr], input: &[FsFr], n: usize, offset: usize, stride: usize) -> Result<(), String> {
    if stride == 0 {
        return Err("Stride cannot be zero".to_string());
    }
    // fr_t *out, const fr_t *in, size_t n, size_t offset, size_t stride

    // size_t k, k2;

    // if (stride == 0) return C_KZG_BADARGS;

    let k = n / stride;
    let k2 = k * 2;
    // k = n / stride;
    // k2 = k * 2;

    out[0] = input[n - 1 - offset];
    // out[0] = in[n - 1 - offset];
    // for i in 1..
    {
        let mut i = 1;
        while i <= k + 1 && i < k2 {
            out[i] = FsFr::zero();
            i += 1;
        }
    };

    {
        let mut i = k + 2;
        let mut j = 2 * stride - offset - 1;
        while i < k2 {
            out[i] = input[j];
            i += 1;
            j += stride;
        }
    };
    // for (size_t i = 1; i <= k + 1 && i < k2; i++) {
    //     out[i] = FR_ZERO;
    // }
    // for (size_t i = k + 2, j = 2 * stride - offset - 1; i < k2; i++, j += stride) {
    //     out[i] = in[j];
    // }

    // return C_KZG_OK;
    Ok(())
}

fn g1_ifft(out: &mut [FsG1], input: &[FsG1], s: &FsKZGSettings) -> Result<(), String> {
    if input.len() > FIELD_ELEMENTS_PER_EXT_BLOB || !input.len().is_power_of_two() {
        return Err("Invalid input length".to_string());
    }

    let stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();
    fft_g1_fast(out, &input, 1, &s.fs.reverse_roots_of_unity, stride);

    let inv_len = FsFr::from_u64(input.len() as u64).eucl_inverse();
    for i in 0..input.len() {
        out[i] = out[i].mul(&inv_len);
    }

    Ok(())
}

fn g1_fft(out: &mut [FsG1], input: &[FsG1], s: &FsKZGSettings) -> Result<(), String> {
    if input.len() > FIELD_ELEMENTS_PER_EXT_BLOB || !input.len().is_power_of_two() {
        return Err("Invalid input length".to_string());
    }

    let roots_stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();
    fft_g1_fast(out, input, 1, &s.fs.roots_of_unity, roots_stride);

    Ok(())
}

fn compute_fk20_proofs(proofs: &mut [FsG1], poly: &[FsFr], n: usize, s: &FsKZGSettings) -> Result<(), String> {
    let k = n / FIELD_ELEMENTS_PER_CELL;
    let k2 = k * 2;

    let mut coeffs = vec![vec![FsFr::default(); k]; k2];
    let mut h_ext_fft = vec![FsG1::identity(); k2];
    let mut h = vec![FsG1::identity(); k2];
    let mut toeplitz_coeffs = vec![FsFr::default(); k2];
    let mut toeplitz_coeffs_fft = vec![FsFr::default(); k2];
    
    for i in 0..FIELD_ELEMENTS_PER_CELL {
        toeplitz_coeffs_stride(&mut toeplitz_coeffs, poly, n, i, FIELD_ELEMENTS_PER_CELL)?;
        fr_fft(&mut toeplitz_coeffs_fft, &toeplitz_coeffs, &s.fs)?;
        for j in 0..k2 {
            coeffs[j][i] = toeplitz_coeffs_fft[j];
        }
    }

    for i in 0..k2 {
        g1_linear_combination(&mut h_ext_fft[i], &s.x_ext_fft_columns[i], &coeffs[i], FIELD_ELEMENTS_PER_CELL, None);
    }

    g1_ifft(&mut h, &h_ext_fft, s)?;

    g1_fft(proofs, &h, s)?;

    Ok(())
}

pub fn compute_cells_and_kzg_proofs(cells: Option<&mut [[FsFr; FIELD_ELEMENTS_PER_CELL]]>, proofs: Option<&mut [FsG1]>, blob: &[FsFr], s: &FsKZGSettings) -> Result<(), String> {
    if cells.is_none() && proofs.is_none() {
        return Err("Both cells & proofs cannot be none".to_string());
    }
    
    let poly = blob_to_polynomial::<FsFr, FsPoly>(blob)?;

    let mut poly_monomial = vec![FsFr::zero(); FIELD_ELEMENTS_PER_EXT_BLOB];

    poly_lagrange_to_monomial(&mut poly_monomial[..FIELD_ELEMENTS_PER_BLOB], poly, &s.fs)?;

    // compute cells
    if let Some(cells) = cells {
        let mut data_fr = vec![FsFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

        fr_fft(&mut data_fr, &poly_monomial, &s.fs)?;

        reverse_bit_order(&mut data_fr)?;

        for i in 0..CELLS_PER_EXT_BLOB {
            for j in 0..FIELD_ELEMENTS_PER_CELL {
                let index = i * FIELD_ELEMENTS_PER_CELL + j;
                
                cells[i][j] = data_fr[index];
            }
        }
    };

    // compute proofs
    if let Some(proofs) = proofs {
        compute_fk20_proofs(proofs, &poly_monomial, FIELD_ELEMENTS_PER_BLOB, s)?;
        reverse_bit_order(proofs)?;
    }

    Ok(())
}

fn compute_vanishing_polynomial_from_roots(
    roots: &[FsFr]
) -> Result<Vec<FsFr>, String> {
    if roots.len() == 0 {
        return Err("Roots cannot be empty".to_string())
    }

    let mut poly = Vec::new();
    poly.push(roots[0].negate());

    for i in 1..roots.len() {
        let neg_root = roots[i].negate();

        poly.push(neg_root.add(&poly[i - 1]));

        for j in (1..i).rev() {
            poly[j] = poly[j].mul(&neg_root).add(&poly[j - 1]);
        }
    }

    poly.push(FsFr::one());

    Ok(poly)
}

fn vanishing_polynomial_for_missing_cells(
    missing_cell_indicies: &[usize],
    s: &FsKZGSettings
) -> Result<Vec<FsFr>, String> {
    if missing_cell_indicies.len() == 0 || missing_cell_indicies.len() >= CELLS_PER_EXT_BLOB {
        return Err("Invalid missing cell indicies count".to_string())
    }

    const STRIDE: usize = FIELD_ELEMENTS_PER_EXT_BLOB / CELLS_PER_EXT_BLOB;

    let roots = missing_cell_indicies.iter().map(|i| s.get_roots_of_unity_at(*i * STRIDE)).collect::<Vec<_>>();
    let short_vanishing_poly = compute_vanishing_polynomial_from_roots(&roots)?;

    let mut vanishing_poly = (0..FIELD_ELEMENTS_PER_EXT_BLOB).map(|_| FsFr::zero()).collect::<Vec<_>>();

    for i in 0..short_vanishing_poly.len() {
        vanishing_poly[i * FIELD_ELEMENTS_PER_CELL] = short_vanishing_poly[i];
    }

    Ok(vanishing_poly)
}

fn shift_poly(poly: &mut [FsFr], shift_factor: &FsFr) {
    let mut factor_power = FsFr::one();
    for i in 1..poly.len() {
        factor_power = factor_power.mul(shift_factor);
        poly[i] = poly[i].mul(&factor_power);
    }
}

fn coset_fft(input: &[FsFr], s: &FsKZGSettings) -> Result<Vec<FsFr>, String> {
    if input.len() == 0 {
        return Err("Invalid input length".to_string());
    }

    let mut in_shifted = input.to_vec();
    // TODO: move 7 to constant
    shift_poly(&mut in_shifted, &FsFr::from_u64(7));

    let mut output = vec![FsFr::default(); input.len()];
    fr_fft(&mut output, input, &s.fs)?;

    Ok(output)
}

fn coset_ifft(input: &[FsFr], s: &FsKZGSettings) -> Result<Vec<FsFr>, String> {
    if input.len() == 0 {
        return Err("Invalid input length".to_string());
    }

    let mut output = vec![FsFr::default(); input.len()];
    fr_ifft(&mut output, input, &s.fs)?;

    shift_poly(&mut output, &FsFr::one().div(&FsFr::from_u64(7))?);

    Ok(output)
}

fn recover_cells(
    output: &mut [FsFr],
    cell_indicies: &[usize],
    s: &FsKZGSettings
) -> Result<(), String> {
    let mut missing_cell_indicies = Vec::new();

    let mut cells_brp = output.to_vec();
    reverse_bit_order(&mut cells_brp);

    for i in 0..CELLS_PER_EXT_BLOB {
        if cell_indicies.contains(&i) {
            missing_cell_indicies.push(reverse_bits_limited(CELLS_PER_EXT_BLOB, i));
        }
    }

    let missing_cell_indicies = &missing_cell_indicies[..];

    if missing_cell_indicies.len() <= CELLS_PER_EXT_BLOB  / 2 {
        return Err("Not enough cells".to_string());
    }

    let vanishing_poly_coeff = vanishing_polynomial_for_missing_cells(
        missing_cell_indicies,
        s
    )?;

    let mut vanishing_poly_eval = vec![FsFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

    fr_fft(&mut vanishing_poly_eval, &vanishing_poly_coeff, &s.fs);

    let mut extended_evaluation_times_zero = Vec::with_capacity(FIELD_ELEMENTS_PER_EXT_BLOB);

    for i in 0..FIELD_ELEMENTS_PER_EXT_BLOB {
        if cells_brp[i].is_null() {
            extended_evaluation_times_zero.push(FsFr::zero());
        } else {
            extended_evaluation_times_zero.push(cells_brp[i].mul(&vanishing_poly_eval[i]));
        }
    }

    let mut extended_evaluation_times_zero_coeffs = vec![FsFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];
    fr_ifft(&mut extended_evaluation_times_zero_coeffs, &extended_evaluation_times_zero, &s.fs);

    let mut extended_evaluations_over_coset = coset_fft(&extended_evaluation_times_zero_coeffs, s)?;

    let vanishing_poly_over_coset = coset_fft(&vanishing_poly_coeff, s)?;

    for i in 0..FIELD_ELEMENTS_PER_EXT_BLOB {
        extended_evaluations_over_coset[i] = extended_evaluations_over_coset[i].div(&vanishing_poly_over_coset[i])?;
    }

    let reconstructed_poly_coeff = coset_ifft(&extended_evaluations_over_coset, s)?;

    fr_fft(output, &reconstructed_poly_coeff, &s.fs)?;

    reverse_bit_order(output)?;

    Ok(())
}

pub fn recover_cells_and_kzg_proofs(
    recovered_cells: &mut [[FsFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [FsG1]>,
    cell_indicies: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    s: &FsKZGSettings
) -> Result<(), String> {
    if recovered_cells.len() != CELLS_PER_EXT_BLOB || recovered_proofs.is_some_and(|it| it.len() != CELLS_PER_EXT_BLOB) {
        return Err("Invalid output array length".to_string());
    }

    if cells.len() != cell_indicies.len() {
        return Err("Cell indicies mismatch - cells length must be equal to cell indicies length".to_string());
    }

    if cells.len() > CELLS_PER_EXT_BLOB {
        return Err("Cell length cannot be larger than CELLS_PER_EXT_BLOB".to_string());
    }

    if cells.len() < CELLS_PER_EXT_BLOB / 2 {
        return Err("Impossible to recover - cells length cannot be less than CELLS_PER_EXT_BLOB / 2".to_string());
    }

    for i in 0..cells.len() {
        if cell_indicies[i] >= CELLS_PER_EXT_BLOB {
            return Err(format!("Cell index {i} cannot be larger than CELLS_PER_EXT_BLOB"));
        }
    }

    for i in 0..cells.len() {
        let index = cell_indicies[i];

        for j in 0..FIELD_ELEMENTS_PER_CELL {
            if !recovered_cells[index][j].is_null() {
                return Err("Invalid cell".to_string());
            }
        }

        recovered_cells[index] = cells[i];
    }

    if cells.len() != CELLS_PER_EXT_BLOB {
        recover_cells(recovered_cells.as_flattened_mut(), cell_indicies, s)?;
    }

    if let Some(recovered_proofs) = recovered_proofs {
        poly_lagrange_to_monomial(recovered_cells.as_flattened_mut(), recovered_cells.as_flattened(), &s.fs);
    }

    Ok(())
}