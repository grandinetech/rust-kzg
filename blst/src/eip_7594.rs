use crate::{
    fft_fr::fft_fr_fast,
    fft_g1::fft_g1_fast,
    kzg_proofs::{g1_linear_combination, pairings_verify},
    types::{g1::FsG1, g2::FsG2},
    utils::{deserialize_blob, handle_ckzg_badargs, kzg_settings_to_rust},
};
use kzg::{
    common_utils::{reverse_bit_order, reverse_bits_limited},
    eip_4844::{
        blob_to_polynomial, compute_powers, hash, hash_to_bls_field, Blob, Bytes48, CKZGSettings,
        Cell, KZGProof, BYTES_PER_CELL, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT,
        BYTES_PER_PROOF, CELLS_PER_EXT_BLOB, C_KZG_RET, C_KZG_RET_OK, FIELD_ELEMENTS_PER_BLOB,
        FIELD_ELEMENTS_PER_CELL, FIELD_ELEMENTS_PER_EXT_BLOB,
        RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN,
    },
    Fr, G1Mul, KZGSettings, G1, G2,
};

use crate::types::{
    fft_settings::FsFFTSettings, fr::FsFr, kzg_settings::FsKZGSettings, poly::FsPoly,
};

fn fr_ifft(output: &mut [FsFr], input: &[FsFr], s: &FsFFTSettings) -> Result<(), String> {
    let stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();

    fft_fr_fast(output, input, 1, &s.reverse_roots_of_unity, stride);

    let inv_len = FsFr::from_u64(input.len().try_into().unwrap()).inverse();
    for el in output {
        *el = el.mul(&inv_len);
    }

    Ok(())
}

fn fr_fft(output: &mut [FsFr], input: &[FsFr], s: &FsFFTSettings) -> Result<(), String> {
    let roots_stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();
    fft_fr_fast(output, input, 1, &s.roots_of_unity, roots_stride);

    Ok(())
}

fn poly_lagrange_to_monomial(
    output: &mut [FsFr],
    largrange_poly: &[FsFr],
    s: &FsFFTSettings,
) -> Result<(), String> {
    let mut poly = largrange_poly.to_vec();

    reverse_bit_order(&mut poly)?;

    fr_ifft(output, &poly, s)?;

    Ok(())
}

fn toeplitz_coeffs_stride(
    out: &mut [FsFr],
    input: &[FsFr],
    n: usize,
    offset: usize,
    stride: usize,
) -> Result<(), String> {
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
    fft_g1_fast(out, input, 1, &s.fs.reverse_roots_of_unity, stride);

    let inv_len = FsFr::from_u64(input.len() as u64).eucl_inverse();
    for out in out.iter_mut().take(input.len()) {
        *out = out.mul(&inv_len);
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

fn compute_fk20_proofs(
    proofs: &mut [FsG1],
    poly: &[FsFr],
    n: usize,
    s: &FsKZGSettings,
) -> Result<(), String> {
    let k = n / FIELD_ELEMENTS_PER_CELL;
    let k2 = k * 2;

    let mut coeffs = vec![vec![FsFr::default(); k]; k2];
    let mut h_ext_fft = vec![FsG1::identity(); k2];
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
        g1_linear_combination(
            &mut h_ext_fft[i],
            &s.x_ext_fft_columns[i],
            &coeffs[i],
            FIELD_ELEMENTS_PER_CELL,
            None,
        );
    }

    let mut h = vec![FsG1::identity(); k2];
    g1_ifft(&mut h, &h_ext_fft, s)?;

    for h in h.iter_mut().take(k2).skip(k) {
        *h = FsG1::identity();
    }

    g1_fft(proofs, &h, s)?;

    Ok(())
}

pub fn compute_cells_and_kzg_proofs_rust(
    cells: Option<&mut [[FsFr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [FsG1]>,
    blob: &[FsFr],
    s: &FsKZGSettings,
) -> Result<(), String> {
    if cells.is_none() && proofs.is_none() {
        return Err("Both cells & proofs cannot be none".to_string());
    }

    let poly = blob_to_polynomial::<FsFr, FsPoly>(blob)?;

    let mut poly_monomial = vec![FsFr::zero(); FIELD_ELEMENTS_PER_EXT_BLOB];

    poly_lagrange_to_monomial(
        &mut poly_monomial[..FIELD_ELEMENTS_PER_BLOB],
        &poly.coeffs,
        &s.fs,
    )?;

    // compute cells
    if let Some(cells) = cells {
        fr_fft(cells.as_flattened_mut(), &poly_monomial, &s.fs)?;

        reverse_bit_order(cells.as_flattened_mut())?;
    };

    // compute proofs
    if let Some(proofs) = proofs {
        compute_fk20_proofs(proofs, &poly_monomial, FIELD_ELEMENTS_PER_BLOB, s)?;
        reverse_bit_order(proofs)?;
    }

    Ok(())
}

fn compute_vanishing_polynomial_from_roots(roots: &[FsFr]) -> Result<Vec<FsFr>, String> {
    if roots.is_empty() {
        return Err("Roots cannot be empty".to_string());
    }

    let mut poly = Vec::new();
    poly.push(roots[0].negate());

    for i in 1..roots.len() {
        let neg_root = roots[i].negate();

        poly.push(neg_root.add(&poly[i - 1]));

        for j in (1..i).rev() {
            poly[j] = poly[j].mul(&neg_root).add(&poly[j - 1]);
        }
        poly[0] = poly[0].mul(&neg_root);
    }

    poly.push(FsFr::one());

    Ok(poly)
}

fn vanishing_polynomial_for_missing_cells(
    missing_cell_indicies: &[usize],
    s: &FsKZGSettings,
) -> Result<Vec<FsFr>, String> {
    if missing_cell_indicies.is_empty() || missing_cell_indicies.len() >= CELLS_PER_EXT_BLOB {
        return Err("Invalid missing cell indicies count".to_string());
    }

    const STRIDE: usize = FIELD_ELEMENTS_PER_EXT_BLOB / CELLS_PER_EXT_BLOB;

    let roots = missing_cell_indicies
        .iter()
        .map(|i| s.get_roots_of_unity_at(*i * STRIDE))
        .collect::<Vec<_>>();

    let short_vanishing_poly = compute_vanishing_polynomial_from_roots(&roots)?;

    let mut vanishing_poly = vec![FsFr::zero(); FIELD_ELEMENTS_PER_EXT_BLOB];

    for i in 0..short_vanishing_poly.len() {
        vanishing_poly[i * FIELD_ELEMENTS_PER_CELL] = short_vanishing_poly[i];
    }

    Ok(vanishing_poly)
}

fn shift_poly(poly: &mut [FsFr], shift_factor: &FsFr) {
    let mut factor_power = FsFr::one();
    for coeff in poly.iter_mut().skip(1) {
        factor_power = factor_power.mul(shift_factor);
        *coeff = coeff.mul(&factor_power);
    }
}

fn coset_fft(input: &[FsFr], s: &FsKZGSettings) -> Result<Vec<FsFr>, String> {
    if input.is_empty() {
        return Err("Invalid input length".to_string());
    }

    let mut in_shifted = input.to_vec();
    // TODO: move 7 to constant
    shift_poly(&mut in_shifted, &FsFr::from_u64(7));

    let mut output = vec![FsFr::default(); input.len()];
    fr_fft(&mut output, &in_shifted, &s.fs)?;

    Ok(output)
}

fn coset_ifft(input: &[FsFr], s: &FsKZGSettings) -> Result<Vec<FsFr>, String> {
    if input.is_empty() {
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
    s: &FsKZGSettings,
) -> Result<(), String> {
    let mut missing_cell_indicies = Vec::new();

    let mut cells_brp = output.to_vec();
    reverse_bit_order(&mut cells_brp)?;

    for i in 0..CELLS_PER_EXT_BLOB {
        if !cell_indicies.contains(&i) {
            missing_cell_indicies.push(reverse_bits_limited(CELLS_PER_EXT_BLOB, i));
        }
    }

    let missing_cell_indicies = &missing_cell_indicies[..];

    if missing_cell_indicies.len() > CELLS_PER_EXT_BLOB / 2 {
        return Err("Not enough cells".to_string());
    }

    let vanishing_poly_coeff = vanishing_polynomial_for_missing_cells(missing_cell_indicies, s)?;
    let mut vanishing_poly_eval = vec![FsFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

    fr_fft(&mut vanishing_poly_eval, &vanishing_poly_coeff, &s.fs)?;

    let mut extended_evaluation_times_zero = Vec::with_capacity(FIELD_ELEMENTS_PER_EXT_BLOB);

    for i in 0..FIELD_ELEMENTS_PER_EXT_BLOB {
        if cells_brp[i].is_null() {
            extended_evaluation_times_zero.push(FsFr::zero());
        } else {
            extended_evaluation_times_zero.push(cells_brp[i].mul(&vanishing_poly_eval[i]));
        }
    }

    let mut extended_evaluation_times_zero_coeffs =
        vec![FsFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];
    fr_ifft(
        &mut extended_evaluation_times_zero_coeffs,
        &extended_evaluation_times_zero,
        &s.fs,
    )?;

    let mut extended_evaluations_over_coset = coset_fft(&extended_evaluation_times_zero_coeffs, s)?;

    let vanishing_poly_over_coset = coset_fft(&vanishing_poly_coeff, s)?;

    for i in 0..FIELD_ELEMENTS_PER_EXT_BLOB {
        extended_evaluations_over_coset[i] =
            extended_evaluations_over_coset[i].div(&vanishing_poly_over_coset[i])?;
    }

    let reconstructed_poly_coeff = coset_ifft(&extended_evaluations_over_coset, s)?;

    fr_fft(output, &reconstructed_poly_coeff, &s.fs)?;

    reverse_bit_order(output)?;

    Ok(())
}

pub fn recover_cells_and_kzg_proofs_rust(
    recovered_cells: &mut [[FsFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [FsG1]>,
    cell_indicies: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    s: &FsKZGSettings,
) -> Result<(), String> {
    if recovered_cells.len() != CELLS_PER_EXT_BLOB
        || recovered_proofs
            .as_ref()
            .is_some_and(|it| it.len() != CELLS_PER_EXT_BLOB)
    {
        return Err("Invalid output array length".to_string());
    }

    if cells.len() != cell_indicies.len() {
        return Err(
            "Cell indicies mismatch - cells length must be equal to cell indicies length"
                .to_string(),
        );
    }

    if cells.len() > CELLS_PER_EXT_BLOB {
        return Err("Cell length cannot be larger than CELLS_PER_EXT_BLOB".to_string());
    }

    if cells.len() < CELLS_PER_EXT_BLOB / 2 {
        return Err(
            "Impossible to recover - cells length cannot be less than CELLS_PER_EXT_BLOB / 2"
                .to_string(),
        );
    }

    for cell_index in cell_indicies {
        if *cell_index >= CELLS_PER_EXT_BLOB {
            return Err("Cell index cannot be larger than CELLS_PER_EXT_BLOB".to_string());
        }
    }

    for cell in recovered_cells.iter_mut() {
        for fr in cell {
            *fr = FsFr::null();
        }
    }

    for i in 0..cells.len() {
        let index = cell_indicies[i];

        for j in 0..FIELD_ELEMENTS_PER_CELL {
            if !recovered_cells[index][j].is_null() {
                return Err("Invalid output cell".to_string());
            }
        }

        recovered_cells[index] = cells[i];
    }

    if cells.len() != CELLS_PER_EXT_BLOB {
        recover_cells(recovered_cells.as_flattened_mut(), cell_indicies, s)?;
    }

    #[allow(clippy::redundant_slicing)]
    let recovered_cells = &recovered_cells[..];

    if let Some(recovered_proofs) = recovered_proofs {
        let mut poly = vec![FsFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

        poly_lagrange_to_monomial(&mut poly, recovered_cells.as_flattened(), &s.fs)?;

        compute_fk20_proofs(recovered_proofs, &poly, FIELD_ELEMENTS_PER_BLOB, s)?;

        reverse_bit_order(recovered_proofs)?;
    }

    Ok(())
}

fn deduplicate_commitments(commitments: &mut [FsG1], indicies: &mut [usize], count: &mut usize) {
    if *count == 0 {
        return;
    }

    indicies[0] = 0;
    let mut new_count = 1;

    for i in 1..*count {
        let mut exist = false;
        for j in 0..new_count {
            if commitments[i] == commitments[j] {
                indicies[i] = j;
                exist = true;
                break;
            }
        }

        if !exist {
            commitments[new_count] = commitments[i];
            indicies[i] = new_count;
            new_count += 1;
        }
    }
}

fn compute_r_powers_for_verify_cell_kzg_proof_batch(
    commitments: &[FsG1],
    commitment_indices: &[usize],
    cell_indices: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[FsG1],
) -> Result<Vec<FsFr>, String> {
    if commitment_indices.len() != cells.len()
        || cell_indices.len() != cells.len()
        || proofs.len() != cells.len()
    {
        return Err("Cell count mismatch".to_string());
    }

    let input_size = RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN.len()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<u64>()
        + (commitments.len() * BYTES_PER_COMMITMENT)
        + (cells.len() * size_of::<u64>())
        + (cells.len() * size_of::<u64>())
        + (cells.len() * BYTES_PER_CELL)
        + (cells.len() * BYTES_PER_PROOF);

    let mut bytes = vec![0; input_size];
    bytes[..16].copy_from_slice(&RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN);
    bytes[16..24].copy_from_slice(&(FIELD_ELEMENTS_PER_CELL as u64).to_be_bytes());
    bytes[24..32].copy_from_slice(&(commitments.len() as u64).to_be_bytes());
    bytes[32..40].copy_from_slice(&(cells.len() as u64).to_be_bytes());

    let mut offset = 40;
    for commitment in commitments {
        bytes[offset..(offset + BYTES_PER_COMMITMENT)].copy_from_slice(&commitment.to_bytes());
        offset += BYTES_PER_COMMITMENT;
    }

    for i in 0..cells.len() {
        bytes[offset..(offset + 8)].copy_from_slice(&(commitment_indices[i] as u64).to_be_bytes());
        offset += 8;

        bytes[offset..(offset + 8)].copy_from_slice(&(cell_indices[i] as u64).to_be_bytes());
        offset += 8;

        bytes[offset..(offset + BYTES_PER_CELL)].copy_from_slice(
            &cells[i]
                .iter()
                .flat_map(|fr| fr.to_bytes())
                .collect::<Vec<_>>(),
        );
        offset += BYTES_PER_CELL;

        bytes[offset..(offset + BYTES_PER_PROOF)].copy_from_slice(&(proofs[i].to_bytes()));
        offset += BYTES_PER_PROOF;
    }

    let bytes = &bytes[..];

    if offset != input_size {
        return Err("Failed to create challenge - invalid length".to_string());
    }

    let eval_challenge = hash(bytes);
    let r = hash_to_bls_field(&eval_challenge);

    Ok(compute_powers(&r, cells.len()))
}

fn compute_weighted_sum_of_commitments(
    commitments: &[FsG1],
    commitment_indices: &[usize],
    r_powers: &[FsFr],
) -> FsG1 {
    let mut commitment_weights = vec![FsFr::zero(); commitments.len()];

    for i in 0..r_powers.len() {
        commitment_weights[commitment_indices[i]] =
            commitment_weights[commitment_indices[i]].add(&r_powers[i]);
    }

    let mut sum_of_commitments = FsG1::default();
    g1_linear_combination(
        &mut sum_of_commitments,
        commitments,
        &commitment_weights,
        commitments.len(),
        None,
    );

    sum_of_commitments
}

/**
 * This is a precomputed map of cell index to reverse-bits-limited cell index.
 *
 * for (size_t i = 0; i < CELLS_PER_EXT_BLOB; i++)
 *   printf("%#04llx,\n", reverse_bits_limited(CELLS_PER_EXT_BLOB, i));
 *
 * Because of the way our evaluation domain is defined, we can use CELL_INDICES_RBL to find the
 * coset factor of a cell. In particular, for cell i, its coset factor is
 * roots_of_unity[CELLS_INDICES_RBL[i]].
 */
const CELL_INDICES_RBL: [usize; CELLS_PER_EXT_BLOB] = [
    0x00, 0x40, 0x20, 0x60, 0x10, 0x50, 0x30, 0x70, 0x08, 0x48, 0x28, 0x68, 0x18, 0x58, 0x38, 0x78,
    0x04, 0x44, 0x24, 0x64, 0x14, 0x54, 0x34, 0x74, 0x0c, 0x4c, 0x2c, 0x6c, 0x1c, 0x5c, 0x3c, 0x7c,
    0x02, 0x42, 0x22, 0x62, 0x12, 0x52, 0x32, 0x72, 0x0a, 0x4a, 0x2a, 0x6a, 0x1a, 0x5a, 0x3a, 0x7a,
    0x06, 0x46, 0x26, 0x66, 0x16, 0x56, 0x36, 0x76, 0x0e, 0x4e, 0x2e, 0x6e, 0x1e, 0x5e, 0x3e, 0x7e,
    0x01, 0x41, 0x21, 0x61, 0x11, 0x51, 0x31, 0x71, 0x09, 0x49, 0x29, 0x69, 0x19, 0x59, 0x39, 0x79,
    0x05, 0x45, 0x25, 0x65, 0x15, 0x55, 0x35, 0x75, 0x0d, 0x4d, 0x2d, 0x6d, 0x1d, 0x5d, 0x3d, 0x7d,
    0x03, 0x43, 0x23, 0x63, 0x13, 0x53, 0x33, 0x73, 0x0b, 0x4b, 0x2b, 0x6b, 0x1b, 0x5b, 0x3b, 0x7b,
    0x07, 0x47, 0x27, 0x67, 0x17, 0x57, 0x37, 0x77, 0x0f, 0x4f, 0x2f, 0x6f, 0x1f, 0x5f, 0x3f, 0x7f,
];

fn get_coset_shift_pow_for_cell(
    cell_index: usize,
    settings: &FsKZGSettings,
) -> Result<FsFr, String> {
    /*
     * Get the cell index in reverse-bit order.
     * This index points to this cell's coset factor h_k in the roots_of_unity array.
     */
    let cell_idx_rbl = CELL_INDICES_RBL[cell_index];

    /*
     * Get the index to h_k^n in the roots_of_unity array.
     *
     * Multiplying the index of h_k by n, effectively raises h_k to the n-th power,
     * because advancing in the roots_of_unity array corresponds to increasing exponents.
     */
    let h_k_pow_idx = cell_idx_rbl * FIELD_ELEMENTS_PER_CELL;

    if h_k_pow_idx > FIELD_ELEMENTS_PER_EXT_BLOB {
        return Err("Invalid cell index".to_string());
    }

    /* Get h_k^n using the index */
    Ok(settings.get_roots_of_unity_at(h_k_pow_idx))
}

fn get_inv_coset_shift_for_cell(
    cell_index: usize,
    settings: &FsKZGSettings,
) -> Result<FsFr, String> {
    /*
     * Get the cell index in reverse-bit order.
     * This index points to this cell's coset factor h_k in the roots_of_unity array.
     */
    let cell_index_rbl = CELL_INDICES_RBL[cell_index];

    /*
     * Observe that for every element in roots_of_unity, we can find its inverse by
     * accessing its reflected element.
     *
     * For example, consider a multiplicative subgroup with eight elements:
     *   roots = {w^0, w^1, w^2, ... w^7, w^0}
     * For a root of unity in roots[i], we can find its inverse in roots[-i].
     */
    if cell_index_rbl > FIELD_ELEMENTS_PER_EXT_BLOB {
        return Err("Invalid cell index".to_string());
    }
    let inv_coset_factor_idx = FIELD_ELEMENTS_PER_EXT_BLOB - cell_index_rbl;

    /* Get h_k^{-1} using the index */
    if inv_coset_factor_idx > FIELD_ELEMENTS_PER_EXT_BLOB {
        return Err("Invalid cell index".to_string());
    }

    Ok(settings.get_roots_of_unity_at(inv_coset_factor_idx))
}

fn compute_commitment_to_aggregated_interpolation_poly(
    r_powers: &[FsFr],
    cell_indices: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    s: &FsKZGSettings,
) -> Result<FsG1, String> {
    let mut aggregated_column_cells =
        vec![FsFr::zero(); CELLS_PER_EXT_BLOB * FIELD_ELEMENTS_PER_CELL];

    for (cell_index, column_index) in cell_indices.iter().enumerate() {
        for fr_index in 0..FIELD_ELEMENTS_PER_CELL {
            let original_fr = cells[cell_index][fr_index];

            let scaled_fr = original_fr.mul(&r_powers[cell_index]);

            let array_index = column_index * FIELD_ELEMENTS_PER_CELL + fr_index;
            aggregated_column_cells[array_index] =
                aggregated_column_cells[array_index].add(&scaled_fr);
        }
    }

    let mut is_cell_used = [false; CELLS_PER_EXT_BLOB];

    for cell_index in cell_indices {
        is_cell_used[*cell_index] = true;
    }

    let mut aggregated_interpolation_poly = vec![FsFr::zero(); FIELD_ELEMENTS_PER_CELL];
    let mut column_interpolation_poly = vec![FsFr::default(); FIELD_ELEMENTS_PER_CELL];
    for (i, is_cell_used) in is_cell_used.iter().enumerate() {
        if !is_cell_used {
            continue;
        }

        let index = i * FIELD_ELEMENTS_PER_CELL;

        reverse_bit_order(&mut aggregated_column_cells[index..(index + FIELD_ELEMENTS_PER_CELL)])?;

        fr_ifft(
            &mut column_interpolation_poly,
            &aggregated_column_cells[index..(index + FIELD_ELEMENTS_PER_CELL)],
            &s.fs,
        )?;

        let inv_coset_factor = get_inv_coset_shift_for_cell(i, s)?;

        shift_poly(&mut column_interpolation_poly, &inv_coset_factor);

        for k in 0..FIELD_ELEMENTS_PER_CELL {
            aggregated_interpolation_poly[k] =
                aggregated_interpolation_poly[k].add(&column_interpolation_poly[k]);
        }
    }

    let mut commitment_out = FsG1::default();
    g1_linear_combination(
        &mut commitment_out,
        &s.g1_values_monomial,
        &aggregated_interpolation_poly,
        FIELD_ELEMENTS_PER_CELL,
        None,
    ); // TODO: maybe pass precomputation here?

    Ok(commitment_out)
}

fn computed_weighted_sum_of_proofs(
    proofs: &[FsG1],
    r_powers: &[FsFr],
    cell_indices: &[usize],
    s: &FsKZGSettings,
) -> Result<FsG1, String> {
    let num_cells = proofs.len();

    if r_powers.len() != num_cells || cell_indices.len() != num_cells {
        return Err("Length mismatch".to_string());
    }

    let mut weighted_powers_of_r = Vec::with_capacity(num_cells);
    for i in 0..num_cells {
        let h_k_pow = get_coset_shift_pow_for_cell(cell_indices[i], s)?;

        weighted_powers_of_r.push(r_powers[i].mul(&h_k_pow));
    }

    let mut weighted_proofs_sum_out = FsG1::default();
    g1_linear_combination(
        &mut weighted_proofs_sum_out,
        proofs,
        &weighted_powers_of_r,
        num_cells,
        None,
    );

    Ok(weighted_proofs_sum_out)
}

pub fn verify_cell_kzg_proof_batch_rust(
    commitments: &[FsG1],
    cell_indices: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[FsG1],
    s: &FsKZGSettings,
) -> Result<bool, String> {
    if cells.len() != cell_indices.len() {
        return Err("Cell count mismatch".to_string());
    }

    if commitments.len() != cells.len() {
        return Err("Commitment count mismatch".to_string());
    }

    if proofs.len() != cells.len() {
        return Err("Proof count mismatch".to_string());
    }

    if cells.is_empty() {
        return Ok(true);
    }

    for cell_index in cell_indices {
        if *cell_index >= CELLS_PER_EXT_BLOB {
            return Err("Invalid cell index".to_string());
        }
    }

    for proof in proofs {
        if !proof.is_valid() {
            return Err("Proof is not valid".to_string());
        }
    }

    let mut new_count = commitments.len();
    let mut unique_commitments = commitments.to_vec();
    let mut commitment_indices = vec![0usize; cells.len()];
    deduplicate_commitments(
        &mut unique_commitments,
        &mut commitment_indices,
        &mut new_count,
    );

    for commitment in unique_commitments.iter() {
        if !commitment.is_valid() {
            return Err("Commitment is not valid".to_string());
        }
    }

    let unique_commitments = &unique_commitments[0..new_count];

    let r_powers = compute_r_powers_for_verify_cell_kzg_proof_batch(
        unique_commitments,
        &commitment_indices,
        cell_indices,
        cells,
        proofs,
    )?;

    let mut proof_lincomb = FsG1::default();
    g1_linear_combination(&mut proof_lincomb, proofs, &r_powers, cells.len(), None);

    let final_g1_sum =
        compute_weighted_sum_of_commitments(unique_commitments, &commitment_indices, &r_powers);

    let interpolation_poly_commit =
        compute_commitment_to_aggregated_interpolation_poly(&r_powers, cell_indices, cells, s)?;

    let final_g1_sum = final_g1_sum.sub(&interpolation_poly_commit);

    let weighted_sum_of_proofs =
        computed_weighted_sum_of_proofs(proofs, &r_powers, cell_indices, s)?;

    let final_g1_sum = final_g1_sum.add(&weighted_sum_of_proofs);

    let power_of_s = s.g2_values_monomial[FIELD_ELEMENTS_PER_CELL];

    Ok(pairings_verify(
        &final_g1_sum,
        &FsG2::generator(),
        &proof_lincomb,
        &power_of_s,
    ))
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_cells_and_kzg_proofs(
    cells: *mut Cell,
    proofs: *mut KZGProof,
    blob: *const Blob,
    settings: *const CKZGSettings,
) -> C_KZG_RET {
    let mut cells_rs = if cells.is_null() {
        None
    } else {
        Some(vec![
            [FsFr::default(); FIELD_ELEMENTS_PER_CELL];
            CELLS_PER_EXT_BLOB
        ])
    };
    let mut proofs_rs = if proofs.is_null() {
        None
    } else {
        Some(vec![FsG1::default(); CELLS_PER_EXT_BLOB])
    };

    let blob = handle_ckzg_badargs!(deserialize_blob(blob));
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(&*settings));

    handle_ckzg_badargs!(compute_cells_and_kzg_proofs_rust(
        cells_rs.as_deref_mut(),
        proofs_rs.as_deref_mut(),
        &blob,
        &settings
    ));

    if let Some(cells_rs) = cells_rs {
        let cells = core::slice::from_raw_parts_mut(cells, CELLS_PER_EXT_BLOB);
        for (cell_index, cell) in cells_rs.iter().enumerate() {
            for (fr_index, fr) in cell.iter().enumerate() {
                cells[cell_index].bytes[(fr_index * BYTES_PER_FIELD_ELEMENT)
                    ..((fr_index + 1) * BYTES_PER_FIELD_ELEMENT)]
                    .copy_from_slice(&fr.to_bytes());
            }
        }
    }

    if let Some(proofs_rs) = proofs_rs {
        let proofs = core::slice::from_raw_parts_mut(proofs, CELLS_PER_EXT_BLOB);
        for (proof_index, proof) in proofs_rs.iter().enumerate() {
            proofs[proof_index].bytes.copy_from_slice(&proof.to_bytes());
        }
    }

    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn recover_cells_and_kzg_proofs(
    recovered_cells: *mut Cell,
    recovered_proofs: *mut KZGProof,
    cell_indices: *const u64,
    cells: *const Cell,
    num_cells: u64,
    s: *const CKZGSettings,
) -> C_KZG_RET {
    let mut recovered_cells_rs =
        vec![[FsFr::default(); FIELD_ELEMENTS_PER_CELL]; CELLS_PER_EXT_BLOB];

    let mut recovered_proofs_rs = if recovered_proofs.is_null() {
        None
    } else {
        Some(vec![FsG1::default(); CELLS_PER_EXT_BLOB])
    };

    let cell_indicies = core::slice::from_raw_parts(cell_indices, num_cells as usize)
        .iter()
        .map(|it| *it as usize)
        .collect::<Vec<_>>();
    let cells = handle_ckzg_badargs!(core::slice::from_raw_parts(cells, num_cells as usize)
        .iter()
        .map(|it| -> Result<[FsFr; FIELD_ELEMENTS_PER_CELL], String> {
            it.bytes
                .chunks(BYTES_PER_FIELD_ELEMENT)
                .map(FsFr::from_bytes)
                .collect::<Result<Vec<_>, String>>()
                .and_then(|frs| {
                    frs.try_into()
                        .map_err(|_| "Invalid field element count per cell".to_string())
                })
        })
        .collect::<Result<Vec<_>, String>>());
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(&*s));

    handle_ckzg_badargs!(recover_cells_and_kzg_proofs_rust(
        &mut recovered_cells_rs,
        recovered_proofs_rs.as_deref_mut(),
        &cell_indicies,
        &cells,
        &settings,
    ));

    let recovered_cells = core::slice::from_raw_parts_mut(recovered_cells, CELLS_PER_EXT_BLOB);
    for (cell_c, cell_rs) in recovered_cells.iter_mut().zip(recovered_cells_rs.iter()) {
        cell_c.bytes.copy_from_slice(
            &cell_rs
                .iter()
                .flat_map(|fr| fr.to_bytes())
                .collect::<Vec<_>>(),
        );
    }

    if let Some(recovered_proofs_rs) = recovered_proofs_rs {
        let recovered_proofs =
            core::slice::from_raw_parts_mut(recovered_proofs, CELLS_PER_EXT_BLOB);

        for (proof_c, proof_rs) in recovered_proofs.iter_mut().zip(recovered_proofs_rs.iter()) {
            proof_c.bytes = proof_rs.to_bytes();
        }
    }

    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_cell_kzg_proof_batch(
    ok: *mut bool,
    commitments_bytes: *const Bytes48,
    cell_indices: *const u64,
    cells: *const Cell,
    proofs_bytes: *const Bytes48,
    num_cells: u64,
    s: *const CKZGSettings,
) -> C_KZG_RET {
    let commitments = handle_ckzg_badargs!(core::slice::from_raw_parts(
        commitments_bytes,
        num_cells as usize
    )
    .iter()
    .map(|bytes| FsG1::from_bytes(&bytes.bytes))
    .collect::<Result<Vec<_>, String>>());

    let cell_indices = core::slice::from_raw_parts(cell_indices, num_cells as usize)
        .iter()
        .map(|it| *it as usize)
        .collect::<Vec<_>>();

    let cells = handle_ckzg_badargs!(core::slice::from_raw_parts(cells, num_cells as usize)
        .iter()
        .map(|it| -> Result<[FsFr; FIELD_ELEMENTS_PER_CELL], String> {
            it.bytes
                .chunks(BYTES_PER_FIELD_ELEMENT)
                .map(FsFr::from_bytes)
                .collect::<Result<Vec<_>, String>>()
                .and_then(|frs| {
                    frs.try_into()
                        .map_err(|_| "Invalid field element count per cell".to_string())
                })
        })
        .collect::<Result<Vec<_>, String>>());

    let proofs = handle_ckzg_badargs!(core::slice::from_raw_parts(
        proofs_bytes,
        num_cells as usize
    )
    .iter()
    .map(|bytes| FsG1::from_bytes(&bytes.bytes))
    .collect::<Result<Vec<_>, String>>());

    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(&*s));

    *ok = handle_ckzg_badargs!(verify_cell_kzg_proof_batch_rust(
        &commitments,
        &cell_indices,
        &cells,
        &proofs,
        &settings
    ));

    C_KZG_RET_OK
}
