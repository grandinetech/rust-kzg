////////////////////////////// Trait based implementations of functions for EIP-7594 //////////////////////////////

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    common_utils::{reverse_bit_order, reverse_bits_limited},
    eip_4844::{
        blob_to_polynomial, compute_powers, hash, hash_to_bls_field, BYTES_PER_CELL,
        BYTES_PER_COMMITMENT, BYTES_PER_PROOF, CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_BLOB,
        FIELD_ELEMENTS_PER_CELL, FIELD_ELEMENTS_PER_EXT_BLOB,
        RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN,
    },
    G1Mul, KZGSettings, G2,
};
use crate::{
    FFTFr, FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1LinComb, PairingVerify, Poly, FFTG1, G1,
};

fn fr_ifft<TFr: Fr, TFFTSettings: FFTFr<TFr>>(
    output: &mut [TFr],
    input: &[TFr],
    s: &TFFTSettings,
) -> Result<(), String> {
    output.clone_from_slice(&s.fft_fr(input, true)?);

    Ok(())
}

fn fr_fft<TFr: Fr, TFFTSettings: FFTFr<TFr>>(
    output: &mut [TFr],
    input: &[TFr],
    s: &TFFTSettings,
) -> Result<(), String> {
    output.clone_from_slice(&s.fft_fr(input, false)?);

    Ok(())
}

fn poly_lagrange_to_monomial<TFr: Fr, TFFTSettings: FFTFr<TFr>>(
    output: &mut [TFr],
    largrange_poly: &[TFr],
    s: &TFFTSettings,
) -> Result<(), String> {
    let mut poly = largrange_poly.to_vec();

    reverse_bit_order(&mut poly)?;

    fr_ifft(output, &poly, s)?;

    Ok(())
}

fn toeplitz_coeffs_stride<TFr: Fr>(
    out: &mut [TFr],
    input: &[TFr],
    n: usize,
    offset: usize,
    stride: usize,
) -> Result<(), String> {
    if stride == 0 {
        return Err("Stride cannot be zero".to_string());
    }

    let k = n / stride;
    let k2 = k * 2;

    out[0] = input[n - 1 - offset].clone();
    {
        let mut i = 1;
        while i <= k + 1 && i < k2 {
            out[i] = TFr::zero();
            i += 1;
        }
    };

    {
        let mut i = k + 2;
        let mut j = 2 * stride - offset - 1;
        while i < k2 {
            out[i] = input[j].clone();
            i += 1;
            j += stride;
        }
    };

    Ok(())
}

fn g1_ifft<TG1: G1, TFFTSettings: FFTG1<TG1>>(
    out: &mut [TG1],
    input: &[TG1],
    s: &TFFTSettings,
) -> Result<(), String> {
    if input.len() > FIELD_ELEMENTS_PER_EXT_BLOB || !input.len().is_power_of_two() {
        return Err("Invalid input length".to_string());
    }

    out.clone_from_slice(&s.fft_g1(input, true)?);

    Ok(())
}

fn g1_fft<TG1: G1, TFFTSettings: FFTG1<TG1>>(
    out: &mut [TG1],
    input: &[TG1],
    s: &TFFTSettings,
) -> Result<(), String> {
    if input.len() > FIELD_ELEMENTS_PER_EXT_BLOB || !input.len().is_power_of_two() {
        return Err("Invalid input length".to_string());
    }

    out.clone_from_slice(&s.fft_g1(input, false)?);

    Ok(())
}

fn compute_fk20_proofs<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + G1LinComb<TFr, TG1Fp, TG1Affine>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr> + FFTG1<TG1> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    proofs: &mut [TG1],
    poly: &[TFr],
    n: usize,
    s: &TKZGSettings,
) -> Result<(), String> {
    let k = n / FIELD_ELEMENTS_PER_CELL;
    let k2 = k * 2;

    let mut coeffs = vec![vec![TFr::default(); k]; k2];
    let mut h_ext_fft = vec![TG1::identity(); k2];
    let mut toeplitz_coeffs = vec![TFr::default(); k2];
    let mut toeplitz_coeffs_fft = vec![TFr::default(); k2];

    for i in 0..FIELD_ELEMENTS_PER_CELL {
        toeplitz_coeffs_stride(&mut toeplitz_coeffs, poly, n, i, FIELD_ELEMENTS_PER_CELL)?;
        fr_fft(
            &mut toeplitz_coeffs_fft,
            &toeplitz_coeffs,
            s.get_fft_settings(),
        )?;
        for j in 0..k2 {
            coeffs[j][i] = toeplitz_coeffs_fft[j].clone();
        }
    }

    for i in 0..k2 {
        h_ext_fft[i] = TG1::g1_lincomb(
            s.get_x_ext_fft_column(i),
            &coeffs[i],
            FIELD_ELEMENTS_PER_CELL,
            None,
        );
    }

    let mut h = vec![TG1::identity(); k2];
    g1_ifft(&mut h, &h_ext_fft, s.get_fft_settings())?;

    for h in h.iter_mut().take(k2).skip(k) {
        *h = TG1::identity();
    }

    g1_fft(proofs, &h, s.get_fft_settings())?;

    Ok(())
}

pub fn compute_cells_and_kzg_proofs<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + G1LinComb<TFr, TG1Fp, TG1Affine>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr> + FFTG1<TG1> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    cells: Option<&mut [[TFr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [TG1]>,
    blob: &[TFr],
    s: &TKZGSettings,
) -> Result<(), String> {
    if cells.is_none() && proofs.is_none() {
        return Err("Both cells & proofs cannot be none".to_string());
    }

    let poly = blob_to_polynomial::<TFr, TPoly>(blob)?;

    let mut poly_monomial = vec![TFr::zero(); FIELD_ELEMENTS_PER_EXT_BLOB];

    poly_lagrange_to_monomial(
        &mut poly_monomial[..FIELD_ELEMENTS_PER_BLOB],
        poly.get_coeffs(),
        s.get_fft_settings(),
    )?;

    // compute cells
    if let Some(cells) = cells {
        fr_fft(
            cells.as_flattened_mut(),
            &poly_monomial,
            s.get_fft_settings(),
        )?;

        reverse_bit_order(cells.as_flattened_mut())?;
    };

    // compute proofs
    if let Some(proofs) = proofs {
        compute_fk20_proofs(proofs, &poly_monomial, FIELD_ELEMENTS_PER_BLOB, s)?;
        reverse_bit_order(proofs)?;
    }

    Ok(())
}

fn compute_vanishing_polynomial_from_roots<TFr: Fr>(roots: &[TFr]) -> Result<Vec<TFr>, String> {
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

    poly.push(TFr::one());

    Ok(poly)
}

fn vanishing_polynomial_for_missing_cells<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    missing_cell_indicies: &[usize],
    s: &TKZGSettings,
) -> Result<Vec<TFr>, String> {
    if missing_cell_indicies.is_empty() || missing_cell_indicies.len() >= CELLS_PER_EXT_BLOB {
        return Err("Invalid missing cell indicies count".to_string());
    }

    const STRIDE: usize = FIELD_ELEMENTS_PER_EXT_BLOB / CELLS_PER_EXT_BLOB;

    let roots = missing_cell_indicies
        .iter()
        .map(|i| s.get_roots_of_unity_at(*i * STRIDE))
        .collect::<Vec<_>>();

    let short_vanishing_poly = compute_vanishing_polynomial_from_roots(&roots)?;

    let mut vanishing_poly = vec![TFr::zero(); FIELD_ELEMENTS_PER_EXT_BLOB];

    for i in 0..short_vanishing_poly.len() {
        vanishing_poly[i * FIELD_ELEMENTS_PER_CELL] = short_vanishing_poly[i].clone();
    }

    Ok(vanishing_poly)
}

fn shift_poly<TFr: Fr>(poly: &mut [TFr], shift_factor: &TFr) {
    let mut factor_power = TFr::one();
    for coeff in poly.iter_mut().skip(1) {
        factor_power = factor_power.mul(shift_factor);
        *coeff = coeff.mul(&factor_power);
    }
}

fn coset_fft<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>(
    input: &[TFr],
    s: &TFFTSettings,
) -> Result<Vec<TFr>, String> {
    if input.is_empty() {
        return Err("Invalid input length".to_string());
    }

    let mut in_shifted = input.to_vec();
    // TODO: move 7 to constant
    shift_poly(&mut in_shifted, &TFr::from_u64(7));

    let mut output = vec![TFr::default(); input.len()];
    fr_fft(&mut output, &in_shifted, s)?;

    Ok(output)
}

fn coset_ifft<TFr: Fr, TFFTSettings: FFTFr<TFr>>(
    input: &[TFr],
    s: &TFFTSettings,
) -> Result<Vec<TFr>, String> {
    if input.is_empty() {
        return Err("Invalid input length".to_string());
    }

    let mut output = vec![TFr::default(); input.len()];
    fr_ifft(&mut output, input, s)?;

    shift_poly(&mut output, &TFr::one().div(&TFr::from_u64(7))?);

    Ok(output)
}

fn recover_cells<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr> + FFTG1<TG1>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    output: &mut [TFr],
    cell_indicies: &[usize],
    s: &TKZGSettings,
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
    let mut vanishing_poly_eval = vec![TFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

    fr_fft(
        &mut vanishing_poly_eval,
        &vanishing_poly_coeff,
        s.get_fft_settings(),
    )?;

    let mut extended_evaluation_times_zero = Vec::with_capacity(FIELD_ELEMENTS_PER_EXT_BLOB);

    for i in 0..FIELD_ELEMENTS_PER_EXT_BLOB {
        if cells_brp[i].is_null() {
            extended_evaluation_times_zero.push(TFr::zero());
        } else {
            extended_evaluation_times_zero.push(cells_brp[i].mul(&vanishing_poly_eval[i]));
        }
    }

    let mut extended_evaluation_times_zero_coeffs =
        vec![TFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];
    fr_ifft(
        &mut extended_evaluation_times_zero_coeffs,
        &extended_evaluation_times_zero,
        s.get_fft_settings(),
    )?;

    let mut extended_evaluations_over_coset =
        coset_fft(&extended_evaluation_times_zero_coeffs, s.get_fft_settings())?;

    let vanishing_poly_over_coset = coset_fft(&vanishing_poly_coeff, s.get_fft_settings())?;

    for i in 0..FIELD_ELEMENTS_PER_EXT_BLOB {
        extended_evaluations_over_coset[i] =
            extended_evaluations_over_coset[i].div(&vanishing_poly_over_coset[i])?;
    }

    let reconstructed_poly_coeff =
        coset_ifft(&extended_evaluations_over_coset, s.get_fft_settings())?;

    fr_fft(output, &reconstructed_poly_coeff, s.get_fft_settings())?;

    reverse_bit_order(output)?;

    Ok(())
}

pub fn recover_cells_and_kzg_proofs<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + G1LinComb<TFr, TG1Fp, TG1Affine>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr> + FFTG1<TG1>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    recovered_cells: &mut [[TFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [TG1]>,
    cell_indicies: &[usize],
    cells: &[[TFr; FIELD_ELEMENTS_PER_CELL]],
    s: &TKZGSettings,
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
            *fr = TFr::null();
        }
    }

    for i in 0..cells.len() {
        let index = cell_indicies[i];

        for j in 0..FIELD_ELEMENTS_PER_CELL {
            if !recovered_cells[index][j].is_null() {
                return Err("Invalid output cell".to_string());
            }
        }

        recovered_cells[index] = cells[i].clone();
    }

    if cells.len() != CELLS_PER_EXT_BLOB {
        recover_cells(recovered_cells.as_flattened_mut(), cell_indicies, s)?;
    }

    #[allow(clippy::redundant_slicing)]
    let recovered_cells = &recovered_cells[..];

    if let Some(recovered_proofs) = recovered_proofs {
        let mut poly = vec![TFr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

        poly_lagrange_to_monomial(
            &mut poly,
            recovered_cells.as_flattened(),
            s.get_fft_settings(),
        )?;

        compute_fk20_proofs(recovered_proofs, &poly, FIELD_ELEMENTS_PER_BLOB, s)?;

        reverse_bit_order(recovered_proofs)?;
    }

    Ok(())
}

fn deduplicate_commitments<TG1: PartialEq + Clone>(
    commitments: &mut [TG1],
    indicies: &mut [usize],
    count: &mut usize,
) {
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
            commitments[new_count] = commitments[i].clone();
            indicies[i] = new_count;
            new_count += 1;
        }
    }
}

fn compute_r_powers_for_verify_cell_kzg_proof_batch<TG1: G1, TFr: Fr>(
    commitments: &[TG1],
    commitment_indices: &[usize],
    cell_indices: &[usize],
    cells: &[[TFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[TG1],
) -> Result<Vec<TFr>, String> {
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

fn compute_weighted_sum_of_commitments<
    TG1: G1 + G1LinComb<TFr, TG1Fp, TG1Affine>,
    TFr: Fr,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    commitments: &[TG1],
    commitment_indices: &[usize],
    r_powers: &[TFr],
) -> TG1 {
    let mut commitment_weights = vec![TFr::zero(); commitments.len()];

    for i in 0..r_powers.len() {
        commitment_weights[commitment_indices[i]] =
            commitment_weights[commitment_indices[i]].add(&r_powers[i]);
    }

    TG1::g1_lincomb(commitments, &commitment_weights, commitments.len(), None)
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

fn get_coset_shift_pow_for_cell<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    cell_index: usize,
    settings: &TKZGSettings,
) -> Result<TFr, String> {
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

fn get_inv_coset_shift_for_cell<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    cell_index: usize,
    settings: &TKZGSettings,
) -> Result<TFr, String> {
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

fn compute_commitment_to_aggregated_interpolation_poly<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + G1LinComb<TFr, TG1Fp, TG1Affine>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr> + FFTG1<TG1>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    r_powers: &[TFr],
    cell_indices: &[usize],
    cells: &[[TFr; FIELD_ELEMENTS_PER_CELL]],
    s: &TKZGSettings,
) -> Result<TG1, String> {
    let mut aggregated_column_cells =
        vec![TFr::zero(); CELLS_PER_EXT_BLOB * FIELD_ELEMENTS_PER_CELL];

    for (cell_index, column_index) in cell_indices.iter().enumerate() {
        for fr_index in 0..FIELD_ELEMENTS_PER_CELL {
            let original_fr = cells[cell_index][fr_index].clone();

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

    let mut aggregated_interpolation_poly = vec![TFr::zero(); FIELD_ELEMENTS_PER_CELL];
    let mut column_interpolation_poly = vec![TFr::default(); FIELD_ELEMENTS_PER_CELL];
    for (i, is_cell_used) in is_cell_used.iter().enumerate() {
        if !is_cell_used {
            continue;
        }

        let index = i * FIELD_ELEMENTS_PER_CELL;

        reverse_bit_order(&mut aggregated_column_cells[index..(index + FIELD_ELEMENTS_PER_CELL)])?;

        fr_ifft(
            &mut column_interpolation_poly,
            &aggregated_column_cells[index..(index + FIELD_ELEMENTS_PER_CELL)],
            s.get_fft_settings(),
        )?;

        let inv_coset_factor = get_inv_coset_shift_for_cell(i, s)?;

        shift_poly(&mut column_interpolation_poly, &inv_coset_factor);

        for k in 0..FIELD_ELEMENTS_PER_CELL {
            aggregated_interpolation_poly[k] =
                aggregated_interpolation_poly[k].add(&column_interpolation_poly[k]);
        }
    }

    // TODO: maybe pass precomputation here?
    Ok(TG1::g1_lincomb(
        s.get_g1_monomial(),
        &aggregated_interpolation_poly,
        FIELD_ELEMENTS_PER_CELL,
        None,
    ))
}

fn computed_weighted_sum_of_proofs<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + G1LinComb<TFr, TG1Fp, TG1Affine>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    proofs: &[TG1],
    r_powers: &[TFr],
    cell_indices: &[usize],
    s: &TKZGSettings,
) -> Result<TG1, String> {
    let num_cells = proofs.len();

    if r_powers.len() != num_cells || cell_indices.len() != num_cells {
        return Err("Length mismatch".to_string());
    }

    let mut weighted_powers_of_r = Vec::with_capacity(num_cells);
    for i in 0..num_cells {
        let h_k_pow = get_coset_shift_pow_for_cell(cell_indices[i], s)?;

        weighted_powers_of_r.push(r_powers[i].mul(&h_k_pow));
    }

    Ok(TG1::g1_lincomb(
        proofs,
        &weighted_powers_of_r,
        num_cells,
        None,
    ))
}

pub fn verify_cell_kzg_proof_batch<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + G1LinComb<TFr, TG1Fp, TG1Affine> + PairingVerify<TG1, TG2>,
    TG2: G2,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr> + FFTG1<TG1>,
    TPoly: Poly<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    commitments: &[TG1],
    cell_indices: &[usize],
    cells: &[[TFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[TG1],
    s: &TKZGSettings,
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

    let proof_lincomb = TG1::g1_lincomb(proofs, &r_powers, cells.len(), None);

    let final_g1_sum =
        compute_weighted_sum_of_commitments(unique_commitments, &commitment_indices, &r_powers);

    let interpolation_poly_commit =
        compute_commitment_to_aggregated_interpolation_poly(&r_powers, cell_indices, cells, s)?;

    let final_g1_sum = final_g1_sum.sub(&interpolation_poly_commit);

    let weighted_sum_of_proofs =
        computed_weighted_sum_of_proofs(proofs, &r_powers, cell_indices, s)?;

    let final_g1_sum = final_g1_sum.add(&weighted_sum_of_proofs);

    let power_of_s = &s.get_g2_monomial()[FIELD_ELEMENTS_PER_CELL];

    Ok(TG1::verify(
        &final_g1_sum,
        &TG2::generator(),
        &proof_lincomb,
        power_of_s,
    ))
}
