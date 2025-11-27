use alloc::format;
use core::mem::size_of;
use core::{fmt::Debug, hash::Hash};
use hashbrown::{HashMap, HashSet};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::G1ProjAddAffine;
use crate::{
    cfg_iter, cfg_iter_mut,
    common_utils::{reverse_bit_order, reverse_bits_limited},
    eip_4844::{
        blob_to_polynomial, compute_powers, hash, hash_to_bls_field, BYTES_PER_COMMITMENT,
        BYTES_PER_FIELD_ELEMENT, BYTES_PER_PROOF,
    },
    eth, FFTFr, FFTSettings, Fr, G1Affine, G1Fp, G1LinComb, KZGSettings, PairingVerify, Poly,
    FFTG1, G1, G2,
};

pub const RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN: [u8; 16] = *b"RCKZGCBATCH__V1_";

pub trait EcBackend {
    type Fr: Fr + Debug + Send;
    type G1Fp: G1Fp;
    type G1Affine: G1Affine<Self::G1, Self::G1Fp>;
    type G1ProjAddAffine: G1ProjAddAffine<Self::G1, Self::G1Fp, Self::G1Affine>;
    type G1: G1
        + G1LinComb<Self::Fr, Self::G1Fp, Self::G1Affine, Self::G1ProjAddAffine>
        + PairingVerify<Self::G1, Self::G2>
        + Eq
        + Hash;
    type G2: G2;
    type Poly: Poly<Self::Fr>;
    type FFTSettings: FFTSettings<Self::Fr> + FFTFr<Self::Fr> + FFTG1<Self::G1>;
    type KZGSettings: KZGSettings<
        Self::Fr,
        Self::G1,
        Self::G2,
        Self::FFTSettings,
        Self::Poly,
        Self::G1Fp,
        Self::G1Affine,
        Self::G1ProjAddAffine,
    >;
}

/// Deduplicates a vector and creates a mapping of original indices to deduplicated indices.
///
/// This function is taken from https://github.com/crate-crypto/rust-eth-kzg/blob/63d469ce1c98a9898a0d8cd717aa3ebe46ace227/eip7594/src/verifier.rs#L43-L68
fn deduplicate_with_indices<T: Eq + core::hash::Hash + Clone>(input: &[T]) -> (Vec<T>, Vec<usize>) {
    let mut unique_items = Vec::new();
    let mut index_map = HashMap::new();
    let mut indices = Vec::with_capacity(input.len());

    for item in input {
        let index = match index_map.get(&item) {
            Some(&index) => index,
            None => {
                let new_index = unique_items.len();
                unique_items.push(item.clone());
                index_map.insert(item, new_index);
                new_index
            }
        };
        indices.push(index);
    }

    (unique_items, indices)
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
const CELL_INDICES_RBL: [usize; 128] = [
    0x00, 0x40, 0x20, 0x60, 0x10, 0x50, 0x30, 0x70, 0x08, 0x48, 0x28, 0x68, 0x18, 0x58, 0x38, 0x78,
    0x04, 0x44, 0x24, 0x64, 0x14, 0x54, 0x34, 0x74, 0x0c, 0x4c, 0x2c, 0x6c, 0x1c, 0x5c, 0x3c, 0x7c,
    0x02, 0x42, 0x22, 0x62, 0x12, 0x52, 0x32, 0x72, 0x0a, 0x4a, 0x2a, 0x6a, 0x1a, 0x5a, 0x3a, 0x7a,
    0x06, 0x46, 0x26, 0x66, 0x16, 0x56, 0x36, 0x76, 0x0e, 0x4e, 0x2e, 0x6e, 0x1e, 0x5e, 0x3e, 0x7e,
    0x01, 0x41, 0x21, 0x61, 0x11, 0x51, 0x31, 0x71, 0x09, 0x49, 0x29, 0x69, 0x19, 0x59, 0x39, 0x79,
    0x05, 0x45, 0x25, 0x65, 0x15, 0x55, 0x35, 0x75, 0x0d, 0x4d, 0x2d, 0x6d, 0x1d, 0x5d, 0x3d, 0x7d,
    0x03, 0x43, 0x23, 0x63, 0x13, 0x53, 0x33, 0x73, 0x0b, 0x4b, 0x2b, 0x6b, 0x1b, 0x5b, 0x3b, 0x7b,
    0x07, 0x47, 0x27, 0x67, 0x17, 0x57, 0x37, 0x77, 0x0f, 0x4f, 0x2f, 0x6f, 0x1f, 0x5f, 0x3f, 0x7f,
];

pub trait DAS<B: EcBackend> {
    fn kzg_settings(&self) -> &B::KZGSettings;

    fn recover_cells_and_kzg_proofs(
        &self,
        recovered_cells: &mut [B::Fr],
        recovered_proofs: Option<&mut [B::G1]>,
        cell_indices: &[usize],
        cells: &[B::Fr],
    ) -> Result<(), String> {
        let kzg_settings = self.kzg_settings();
        let ts_len = kzg_settings.get_g1_monomial().len();
        let cell_size = kzg_settings.get_cell_size();

        if recovered_cells.len() != 2 * ts_len
            || recovered_proofs
                .as_ref()
                .is_some_and(|it| it.len() != (2 * ts_len) / cell_size)
        {
            return Err("Invalid output array length".to_string());
        }

        if cells.len() / cell_size != cell_indices.len() {
            return Err(
                "Cell indicies mismatch - cells length must be equal to cell indicies length"
                    .to_string(),
            );
        }

        if cells.len() > 2 * ts_len {
            return Err("Cell length cannot be larger than CELLS_PER_EXT_BLOB".to_string());
        }

        if cells.len() < ts_len {
            return Err(
                "Impossible to recover - cells length cannot be less than CELLS_PER_EXT_BLOB / 2"
                    .to_string(),
            );
        }

        for fr in recovered_cells.iter_mut() {
            *fr = B::Fr::null();
        }

        // Trick to use HashSet, to check for duplicate commitments, is taken from rust-eth-kzg:
        // https://github.com/crate-crypto/rust-eth-kzg/blob/63d469ce1c98a9898a0d8cd717aa3ebe46ace227/eip7594/src/recovery.rs#L64-L76
        let mut provided_indices = HashSet::new();
        for ((i, &cell_index), next_cell_index) in cell_indices
            .iter()
            .enumerate()
            .zip(cell_indices.iter().map(Some).skip(1).chain(Some(None)))
        {
            if cell_index >= (2 * ts_len) / cell_size {
                return Err(format!("Invalid cell index {cell_index}, position {i}: Cell index cannot be larger than CELLS_PER_EXT_BLOB"));
            }

            if let Some(&idx) = next_cell_index {
                if idx <= cell_index {
                    return Err(
                        format!("Invalid cell indices: Indices must be in strictly ascending order, but indices at positions {i} and {next} are not ({cell_index} >= {idx})", next = i + 1)
                    );
                }
            }

            if !provided_indices.insert(cell_index) {
                return Err(format!(
                    "Invalid cell indices: cell index {cell_index} appears twice."
                ));
            }

            recovered_cells[cell_index * cell_size..(cell_index + 1) * cell_size]
                .clone_from_slice(&cells[i * cell_size..(i + 1) * cell_size]);
        }

        let fft_settings = kzg_settings.get_fft_settings();

        if cells.len() != 2 * ts_len {
            recover_cells::<B>(
                cell_size,
                recovered_cells,
                &provided_indices,
                fft_settings,
                2 * ts_len,
            )
            .map_err(|err| format!("Cell recovery failed with error: {err}"))?;
        }

        #[allow(clippy::redundant_slicing)]
        let recovered_cells = &recovered_cells[..];

        if let Some(recovered_proofs) = recovered_proofs {
            let mut poly = vec![B::Fr::default(); ts_len * 2];
            poly.clone_from_slice(recovered_cells);
            poly_lagrange_to_monomial::<B>(&mut poly, fft_settings)?;

            let res = compute_fk20_proofs::<B>(
                cell_size,
                &poly,
                ts_len,
                fft_settings,
                self.kzg_settings(),
            )
            .map_err(|err| format!("Proof computation failed with error: {err}"))?;
            recovered_proofs.clone_from_slice(&res);

            reverse_bit_order(recovered_proofs)?;
        }

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn recover_cells_and_kzg_proofs_batch(
        &self,
        cell_indices: &[Vec<usize>],
        cells: &[Vec<B::Fr>],
    ) -> Result<(Vec<Vec<B::Fr>>, Vec<Vec<B::G1>>), String>
    where
        Self: Sync,
    {
        if cell_indices.len() != cells.len() {
            return Err("Cell indices & cells mismatch".to_string());
        }

        let kzg_settings = self.kzg_settings();
        let ts_len = kzg_settings.get_g1_monomial().len();
        let cell_size = kzg_settings.get_cell_size();

        cfg_iter!(cells)
            .zip(cfg_iter!(cell_indices))
            .map(|(cells, cell_indices)| {
                let mut recovered_cells = vec![B::Fr::null(); 2 * ts_len];
                let mut recovered_proofs = vec![B::G1::default(); (2 * ts_len) / cell_size];

                self.recover_cells_and_kzg_proofs(
                    &mut recovered_cells,
                    Some(&mut recovered_proofs),
                    cell_indices,
                    cells,
                )?;

                Ok((recovered_cells, recovered_proofs))
            })
            .collect::<Result<(Vec<_>, Vec<_>), String>>()
    }

    fn compute_cells_and_kzg_proofs(
        &self,
        cells: Option<&mut [B::Fr]>,
        proofs: Option<&mut [B::G1]>,
        blob: &[B::Fr],
    ) -> Result<(), String> {
        if cells.is_none() && proofs.is_none() {
            return Err("Both cells & proofs cannot be none".to_string());
        }

        let settings = self.kzg_settings();
        let ts_size = settings.get_g1_monomial().len();
        let cell_size = settings.get_cell_size();

        let poly = blob_to_polynomial::<B::Fr, B::Poly>(blob)?;

        let mut poly_monomial = vec![B::Fr::zero(); 2 * ts_size];
        poly_monomial[0..ts_size].clone_from_slice(poly.get_coeffs());

        let fft_settings = self.kzg_settings().get_fft_settings();
        poly_lagrange_to_monomial::<B>(&mut poly_monomial[..ts_size], fft_settings)?;

        // compute cells
        if let Some(cells) = cells {
            cells.clone_from_slice(
                &fft_settings
                    .fft_fr(&poly_monomial, false)
                    .map_err(|err| format!("Cell computation failed with error: {err}"))?,
            );

            reverse_bit_order(cells)?;
        };

        // compute proofs
        if let Some(proofs) = proofs {
            let result = compute_fk20_proofs::<B>(
                cell_size,
                &poly_monomial,
                ts_size,
                fft_settings,
                settings,
            )
            .map_err(|err| format!("Proof computation failed with error: {err:?}"))?;
            proofs.clone_from_slice(&result);
            reverse_bit_order(proofs)?;
        }

        Ok(())
    }

    fn verify_cell_kzg_proof_batch(
        &self,
        commitments: &[B::G1],
        cell_indices: &[usize],
        cells: &[B::Fr],
        proofs: &[B::G1],
    ) -> Result<bool, String> {
        let settings = self.kzg_settings();
        let cell_size = settings.get_cell_size();
        let cell_count = cells.len() / cell_size;
        let ts_size = settings.get_g1_monomial().len();

        if cells.len() != cell_indices.len() * cell_size {
            return Err("Cell count mismatch".to_string());
        }

        if commitments.len() != cell_count {
            return Err("Commitment count mismatch".to_string());
        }

        if proofs.len() != cell_count {
            return Err("Proof count mismatch".to_string());
        }

        if cells.is_empty() {
            return Ok(true);
        }

        if cfg_iter!(cell_indices).any(|&cell_index| cell_index >= (2 * ts_size) / cell_size) {
            return Err("Invalid cell index".to_string());
        }

        if cfg_iter!(proofs).any(|proof| !proof.is_valid()) {
            return Err("Proof is not valid".to_string());
        }

        let (unique_commitments, commitment_indices) = deduplicate_with_indices(commitments);

        if cfg_iter!(unique_commitments).any(|commitment| !commitment.is_valid()) {
            return Err("Commitment is not valid".to_string());
        }

        let fft_settings = settings.get_fft_settings();

        let r = Self::compute_verify_cell_kzg_proof_batch_challenge(
            cell_size,
            &unique_commitments,
            &commitment_indices,
            cell_indices,
            cells,
            proofs,
            ts_size,
        )?;

        let r_powers = compute_powers(&r, cell_count);

        let proof_lincomb = B::G1::g1_lincomb(proofs, &r_powers, cell_count, None);

        let final_g1_sum = compute_weighted_sum_of_commitments::<B>(
            &unique_commitments,
            &commitment_indices,
            &r_powers,
        );

        let interpolation_poly_commit = compute_commitment_to_aggregated_interpolation_poly::<B>(
            cell_size,
            &r_powers,
            cell_indices,
            cells,
            fft_settings,
            settings.get_g1_monomial(),
        )?;

        let final_g1_sum = final_g1_sum.sub(&interpolation_poly_commit);

        let weighted_sum_of_proofs = computed_weighted_sum_of_proofs::<B>(
            cell_size,
            proofs,
            &r_powers,
            cell_indices,
            fft_settings,
            ts_size * 2,
        )?;

        let final_g1_sum = final_g1_sum.add(&weighted_sum_of_proofs);

        let power_of_s = &settings.get_g2_monomial()[cell_size];

        Ok(B::G1::verify(
            &final_g1_sum,
            &B::G2::generator(),
            &proof_lincomb,
            power_of_s,
        ))
    }

    fn compute_verify_cell_kzg_proof_batch_challenge(
        cell_size: usize,
        commitments: &[B::G1],
        commitment_indices: &[usize],
        cell_indices: &[usize],
        cells: &[B::Fr],
        proofs: &[B::G1],
        blob_size: usize,
    ) -> Result<B::Fr, String> {
        let cell_count = cells.len() / cell_size;

        if commitment_indices.len() != cell_count
            || cell_indices.len() != cell_count
            || proofs.len() != cell_count
        {
            return Err("Cell count mismatch".to_string());
        }

        let input_size = RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN.len() /* the domain separator */
            + size_of::<u64>()                                               /* field elements per blob */
            + size_of::<u64>()                                               /* cell_size */
            + size_of::<u64>()                                               /* commitment count */
            + size_of::<u64>()                                               /* cell count */
            + (commitments.len() * BYTES_PER_COMMITMENT)                     /* commitment bytes */
            + (cell_count * size_of::<u64>())                                /* commitment indices */
            + (cell_count * size_of::<u64>())                                /* cell indices */
            + (cell_count * cell_size * BYTES_PER_FIELD_ELEMENT)             /* cells */
            + (cell_count * BYTES_PER_PROOF)                                 /* proofs bytes */
            ;

        let mut bytes = Vec::with_capacity(input_size);
        bytes.extend_from_slice(&RANDOM_CHALLENGE_KZG_CELL_BATCH_DOMAIN);
        bytes.extend_from_slice(&(blob_size as u64).to_be_bytes());
        bytes.extend_from_slice(&(cell_size as u64).to_be_bytes());
        bytes.extend_from_slice(&(commitments.len() as u64).to_be_bytes());
        bytes.extend_from_slice(&(cell_count as u64).to_be_bytes());

        for commitment in commitments {
            bytes.extend_from_slice(&commitment.to_bytes());
        }

        for i in 0..cell_count {
            bytes.extend_from_slice(&(commitment_indices[i] as u64).to_be_bytes());
            bytes.extend_from_slice(&(cell_indices[i] as u64).to_be_bytes());

            for fr in &cells[(i * cell_size)..((i + 1) * cell_size)] {
                bytes.extend_from_slice(&fr.to_bytes());
            }

            bytes.extend_from_slice(&(proofs[i].to_bytes()));
        }

        let bytes = &bytes[..];

        if bytes.len() != input_size {
            return Err("Failed to create challenge - invalid length".to_string());
        }

        let eval_challenge = hash(bytes);
        let r = hash_to_bls_field(&eval_challenge);

        Ok(r)
    }
}

fn shift_poly<B: EcBackend>(poly: &mut [B::Fr], shift_factor: &B::Fr) {
    let mut factor_power = B::Fr::one();
    for coeff in poly.iter_mut().skip(1) {
        factor_power = factor_power.mul(shift_factor);
        *coeff = coeff.mul(&factor_power);
    }
}

fn coset_fft<B: EcBackend>(
    mut input: Vec<B::Fr>,
    fft_settings: &B::FFTSettings,
) -> Result<Vec<B::Fr>, String> {
    if input.is_empty() {
        return Err("Invalid input length".to_string());
    }

    // TODO: move 7 to constant
    shift_poly::<B>(&mut input, &B::Fr::from_u64(7));

    fft_settings.fft_fr(&input, false)
}

fn coset_ifft<B: EcBackend>(
    input: &[B::Fr],
    fft_settings: &B::FFTSettings,
) -> Result<Vec<B::Fr>, String> {
    if input.is_empty() {
        return Err("Invalid input length".to_string());
    }

    let mut output = fft_settings.fft_fr(input, true)?;

    // TODO: move 1/7 to constant
    shift_poly::<B>(&mut output, &B::Fr::one().div(&B::Fr::from_u64(7))?);

    Ok(output)
}

fn compute_vanishing_polynomial_from_roots<B: EcBackend>(
    roots: &[B::Fr],
) -> Result<Vec<B::Fr>, String> {
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

    poly.push(B::Fr::one());

    Ok(poly)
}

fn vanishing_polynomial_for_missing_cells<B: EcBackend>(
    cell_size: usize,
    missing_cell_indicies: &[usize],
    fft_settings: &B::FFTSettings,
    field_elements_per_ext_blob: usize,
) -> Result<Vec<B::Fr>, String> {
    let cells_per_ext_blob = field_elements_per_ext_blob / cell_size;

    if missing_cell_indicies.is_empty() || missing_cell_indicies.len() >= cells_per_ext_blob {
        return Err("Invalid missing cell indicies count".to_string());
    }

    let stride = field_elements_per_ext_blob / cells_per_ext_blob;

    let roots = missing_cell_indicies
        .iter()
        .map(|i| fft_settings.get_roots_of_unity_at(*i * stride))
        .collect::<Vec<_>>();

    let short_vanishing_poly = compute_vanishing_polynomial_from_roots::<B>(&roots)?;

    let mut vanishing_poly = vec![B::Fr::zero(); field_elements_per_ext_blob];

    for (i, coeff) in short_vanishing_poly.into_iter().enumerate() {
        vanishing_poly[i * cell_size] = coeff
    }

    Ok(vanishing_poly)
}

fn recover_cells<B: EcBackend>(
    cell_size: usize,
    output: &mut [B::Fr],
    cell_indicies: &HashSet<usize>,
    fft_settings: &B::FFTSettings,
    field_elements_per_ext_blob: usize,
) -> Result<(), String> {
    let cells_per_ext_blob = field_elements_per_ext_blob / cell_size;
    let mut missing_cell_indicies = Vec::new();

    let mut cells_brp = output.to_vec();
    reverse_bit_order(&mut cells_brp)?;

    for i in 0..cells_per_ext_blob {
        if !cell_indicies.contains(&i) {
            missing_cell_indicies.push(reverse_bits_limited(cells_per_ext_blob, i));
        }
    }

    let missing_cell_indicies = &missing_cell_indicies[..];

    if missing_cell_indicies.len() > cells_per_ext_blob / 2 {
        return Err("Not enough cells".to_string());
    }

    let vanishing_poly_coeff = vanishing_polynomial_for_missing_cells::<B>(
        cell_size,
        missing_cell_indicies,
        fft_settings,
        field_elements_per_ext_blob,
    )?;

    let vanishing_poly_eval = fft_settings
        .fft_fr(&vanishing_poly_coeff, false)
        .map_err(|err| format!("Vanishing polynomial evaluation failed: {err}"))?;

    let mut extended_evaluation_times_zero = Vec::with_capacity(field_elements_per_ext_blob);

    for i in 0..field_elements_per_ext_blob {
        if cells_brp[i].is_null() {
            extended_evaluation_times_zero.push(B::Fr::zero());
        } else {
            extended_evaluation_times_zero.push(cells_brp[i].mul(&vanishing_poly_eval[i]));
        }
    }

    let extended_evaluation_times_zero_coeffs =
        fft_settings.fft_fr(&extended_evaluation_times_zero, true)?;
    let mut extended_evaluations_over_coset =
        coset_fft::<B>(extended_evaluation_times_zero_coeffs, fft_settings)?;

    let mut vanishing_poly_over_coset = coset_fft::<B>(vanishing_poly_coeff, fft_settings)?;
    batch_inverse::<B>(&mut vanishing_poly_over_coset);

    for i in 0..field_elements_per_ext_blob {
        extended_evaluations_over_coset[i] =
            extended_evaluations_over_coset[i].mul(&vanishing_poly_over_coset[i]);
    }

    let reconstructed_poly_coeff = coset_ifft::<B>(&extended_evaluations_over_coset, fft_settings)?;

    let out = fft_settings.fft_fr(&reconstructed_poly_coeff, false)?;
    output.clone_from_slice(&out);

    reverse_bit_order(output)?;

    Ok(())
}

fn poly_lagrange_to_monomial<B: EcBackend>(
    lagrange_poly: &mut [B::Fr],
    fft_settings: &B::FFTSettings,
) -> Result<(), String> {
    let mut poly = lagrange_poly.to_vec();

    reverse_bit_order(&mut poly)?;

    lagrange_poly.clone_from_slice(&fft_settings.fft_fr(&poly, true)?);

    Ok(())
}

fn toeplitz_coeffs_stride<B: EcBackend>(
    out: &mut [B::Fr],
    input: &[B::Fr],
    n: usize,
    offset: usize,
    stride: usize,
) -> Result<(), String> {
    let r = n / stride;
    let l = stride;
    let d = n - 1;
    let d_minus_i = d - offset;

    if d < offset {
        return Err("Invalid offset".to_string());
    }

    for j in out.iter_mut() {
        *j = B::Fr::zero();
    }

    out[0] = input[d_minus_i].clone();

    for j in 1..r - 1 {
        out[2 * r - j] = input[d_minus_i - j * l].clone();
    }

    Ok(())
}

fn compute_fk20_proofs<B: EcBackend>(
    cell_size: usize,
    poly: &[B::Fr],
    n: usize,
    fft_settings: &B::FFTSettings,
    kzg_settings: &B::KZGSettings,
) -> Result<Vec<B::G1>, String> {
    let k = n / cell_size;
    let k2 = k * 2;

    let mut coeffs = vec![vec![B::Fr::default(); k]; k2];
    let mut toeplitz_coeffs = vec![B::Fr::default(); k2];
    let mut toeplitz_coeffs_fft = vec![B::Fr::default(); k2];

    for i in 0..cell_size {
        toeplitz_coeffs_stride::<B>(&mut toeplitz_coeffs, poly, n, i, cell_size)?;
        toeplitz_coeffs_fft.clone_from_slice(&fft_settings.fft_fr(&toeplitz_coeffs, false)?);
        for j in 0..k2 {
            coeffs[j][i] = toeplitz_coeffs_fft[j].clone();
        }
    }

    let h_ext_fft = B::G1::g1_lincomb_batch(
        kzg_settings.get_x_ext_fft_columns(),
        &coeffs,
        kzg_settings.get_precomputation(),
    )?;

    let mut h = fft_settings.fft_g1(&h_ext_fft, true)?;

    cfg_iter_mut!(h)
        .take(k2)
        .skip(k)
        .for_each(|h| *h = B::G1::identity());

    fft_settings.fft_g1(&h, false)
}

fn compute_weighted_sum_of_commitments<B: EcBackend>(
    commitments: &[B::G1],
    commitment_indices: &[usize],
    r_powers: &[B::Fr],
) -> B::G1 {
    let mut commitment_weights = vec![B::Fr::zero(); commitments.len()];

    #[cfg(feature = "parallel")]
    {
        let num_threads = rayon::current_num_threads();
        let chunk_size = r_powers.len().div_ceil(num_threads);

        let intermediate_weights: Vec<_> = r_powers
            .par_chunks(chunk_size)
            .zip(commitment_indices.par_chunks(chunk_size))
            .map(|(r_chunk, idx_chunk)| {
                let mut local_weights = vec![B::Fr::zero(); commitments.len()];
                for (r_power, &index) in r_chunk.iter().zip(idx_chunk.iter()) {
                    local_weights[index] = local_weights[index].add(r_power);
                }
                local_weights
            })
            .collect();

        for local_weights in intermediate_weights {
            for (i, weight) in local_weights.into_iter().enumerate() {
                commitment_weights[i] = commitment_weights[i].add(&weight);
            }
        }
    }

    #[cfg(not(feature = "parallel"))]
    {
        for i in 0..r_powers.len() {
            commitment_weights[commitment_indices[i]] =
                commitment_weights[commitment_indices[i]].add(&r_powers[i]);
        }
    }

    B::G1::g1_lincomb(commitments, &commitment_weights, commitments.len(), None)
}

fn get_inv_coset_shift_for_cell<B: EcBackend>(
    cell_size: usize,
    cell_index: usize,
    fft_settings: &B::FFTSettings,
    field_elements_per_ext_blob: usize,
) -> Result<B::Fr, String> {
    let cells_per_ext_blob = field_elements_per_ext_blob / cell_size;
    /*
     * Get the cell index in reverse-bit order.
     * This index points to this cell's coset factor h_k in the roots_of_unity array.
     */
    let cell_index_rbl = if cells_per_ext_blob == eth::CELLS_PER_EXT_BLOB {
        CELL_INDICES_RBL[cell_index]
    } else {
        reverse_bits_limited(cells_per_ext_blob, cell_index)
    };

    /*
     * Observe that for every element in roots_of_unity, we can find its inverse by
     * accessing its reflected element.
     *
     * For example, consider a multiplicative subgroup with eight elements:
     *   roots = {w^0, w^1, w^2, ... w^7, w^0}
     * For a root of unity in roots[i], we can find its inverse in roots[-i].
     */
    if cell_index_rbl > field_elements_per_ext_blob {
        return Err("Invalid cell index".to_string());
    }
    let inv_coset_factor_idx = field_elements_per_ext_blob - cell_index_rbl;

    /* Get h_k^{-1} using the index */
    if inv_coset_factor_idx > field_elements_per_ext_blob {
        return Err("Invalid cell index".to_string());
    }

    Ok(fft_settings.get_roots_of_unity_at(inv_coset_factor_idx))
}

fn compute_commitment_to_aggregated_interpolation_poly<B: EcBackend>(
    cell_size: usize,
    r_powers: &[B::Fr],
    cell_indices: &[usize],
    cells: &[B::Fr],
    fft_settings: &B::FFTSettings,
    g1_monomial: &[B::G1],
) -> Result<B::G1, String> {
    let cells_per_ext_blob = g1_monomial.len() * 2 / cell_size;

    let mut aggregated_column_cells = vec![B::Fr::zero(); cells_per_ext_blob * cell_size];

    for (cell_index, column_index) in cell_indices.iter().enumerate() {
        for fr_index in 0..cell_size {
            let original_fr = cells[cell_index * cell_size + fr_index].clone();

            let scaled_fr = original_fr.mul(&r_powers[cell_index]);

            let array_index = column_index * cell_size + fr_index;
            aggregated_column_cells[array_index] =
                aggregated_column_cells[array_index].add(&scaled_fr);
        }
    }

    let mut is_cell_used = vec![false; cells_per_ext_blob];

    for cell_index in cell_indices {
        is_cell_used[*cell_index] = true;
    }

    let mut aggregated_interpolation_poly = vec![B::Fr::zero(); cell_size];
    for (i, is_cell_used) in is_cell_used.iter().enumerate() {
        if !is_cell_used {
            continue;
        }

        let index = i * cell_size;

        reverse_bit_order(&mut aggregated_column_cells[index..(index + cell_size)])?;

        let mut column_interpolation_poly =
            fft_settings.fft_fr(&aggregated_column_cells[index..(index + cell_size)], true)?;

        let inv_coset_factor =
            get_inv_coset_shift_for_cell::<B>(cell_size, i, fft_settings, g1_monomial.len() * 2)?;

        shift_poly::<B>(&mut column_interpolation_poly, &inv_coset_factor);

        for k in 0..cell_size {
            aggregated_interpolation_poly[k] =
                aggregated_interpolation_poly[k].add(&column_interpolation_poly[k]);
        }
    }

    // TODO: maybe pass precomputation here?
    Ok(B::G1::g1_lincomb(
        g1_monomial,
        &aggregated_interpolation_poly,
        cell_size,
        None,
    ))
}

fn get_coset_shift_pow_for_cell<B: EcBackend>(
    cell_size: usize,
    cell_index: usize,
    fft_settings: &B::FFTSettings,
    field_elements_per_ext_blob: usize,
) -> Result<B::Fr, String> {
    let cells_per_ext_blob = field_elements_per_ext_blob / cell_size;
    /*
     * Get the cell index in reverse-bit order.
     * This index points to this cell's coset factor h_k in the roots_of_unity array.
     */
    let cell_idx_rbl = if cells_per_ext_blob == eth::CELLS_PER_EXT_BLOB {
        CELL_INDICES_RBL[cell_index]
    } else {
        reverse_bits_limited(cells_per_ext_blob, cell_index)
    };

    /*
     * Get the index to h_k^n in the roots_of_unity array.
     *
     * Multiplying the index of h_k by n, effectively raises h_k to the n-th power,
     * because advancing in the roots_of_unity array corresponds to increasing exponents.
     */
    let h_k_pow_idx = cell_idx_rbl * cell_size;

    if h_k_pow_idx > field_elements_per_ext_blob {
        return Err("Invalid cell index".to_string());
    }

    /* Get h_k^n using the index */
    Ok(fft_settings.get_roots_of_unity_at(h_k_pow_idx))
}

fn computed_weighted_sum_of_proofs<B: EcBackend>(
    cell_size: usize,
    proofs: &[B::G1],
    r_powers: &[B::Fr],
    cell_indices: &[usize],
    fft_settings: &B::FFTSettings,
    field_elements_per_ext_blob: usize,
) -> Result<B::G1, String> {
    let num_cells = proofs.len();

    if r_powers.len() != num_cells || cell_indices.len() != num_cells {
        return Err("Length mismatch".to_string());
    }

    let mut weighted_powers_of_r = Vec::with_capacity(num_cells);
    for i in 0..num_cells {
        let h_k_pow = get_coset_shift_pow_for_cell::<B>(
            cell_size,
            cell_indices[i],
            fft_settings,
            field_elements_per_ext_blob,
        )?;

        weighted_powers_of_r.push(r_powers[i].mul(&h_k_pow));
    }

    Ok(B::G1::g1_lincomb(
        proofs,
        &weighted_powers_of_r,
        num_cells,
        None,
    ))
}

/// This function is taken from rust-eth-kzg:
/// https://github.com/crate-crypto/rust-eth-kzg/blob/63d469ce1c98a9898a0d8cd717aa3ebe46ace227/cryptography/bls12_381/src/batch_inversion.rs#L4-L50
fn batch_inverse<B: EcBackend>(v: &mut [B::Fr]) {
    let mut scratch_pad = Vec::with_capacity(v.len());

    let mut tmp = B::Fr::one();
    for f in v.iter() {
        tmp = tmp.mul(f);
        scratch_pad.push(tmp.clone());
    }

    tmp = tmp.inverse();

    for (f, s) in v
        .iter_mut()
        .rev()
        .zip(scratch_pad.iter().rev().skip(1).chain(Some(&B::Fr::one())))
    {
        let new_tmp = tmp.mul(f);
        *f = tmp.mul(s);
        tmp = new_tmp;
    }
}

/*
 * Automatically implement DAS for all backends
 */
impl<B: EcBackend> DAS<B> for B::KZGSettings {
    fn kzg_settings(&self) -> &<B as EcBackend>::KZGSettings {
        self
    }
}
