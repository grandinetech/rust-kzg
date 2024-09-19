////////////////////////////// Trait based implementations of functions for EIP-7594 //////////////////////////////

use crate::{
    common_utils::reverse_bit_order,
    eip_4844::{blob_to_polynomial, FIELD_ELEMENTS_PER_BLOB},
    FFTFr, FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings, PairingVerify, Poly, G1,
    G2,
};

pub fn compute_cells_and_kzg_proofs_rust<
    TFr: Fr,
    TPoly: Poly<TFr>,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + PairingVerify<TG1, TG2>,
    TG2: G2,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
>(
    blob: &[TFr],
    s: &TKZGSettings,
) -> Result<(Vec<TFr>, Vec<TG1>), String> {
    // Ensure blob length is equal to Bytes per blob
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Blob length must be FIELD_ELEMENTS_PER_BLOB"));
    }
    // Convert the blob to a polynomial.
    let polynomial: TPoly = blob_to_polynomial(blob)?;

    // poly_lagrange_to_monomial(polynomial, )

    // Allocate arrays to hold cells and proofs
    let mut cells = vec![TFr::default(); CELLS_PER_EXT_BLOB];
    let mut proofs = vec![TFr::default(); CELLS_PER_EXT_BLOB];

    // Compute cells
    let mut data_fr = vec![TFr::zero(); FIELD_ELEMENTS_PER_EXT_BLOB];

    // Perform FFT on the polynomial
    data_fr = s
        .get_fft_settings()
        .fft_fr(&polynomial.get_coeffs(), false)?;

    // Perform bit reversal permutation
    reverse_bit_order(&mut data_fr)?;

    // Covert field elements to cell bytes
    for (i, cell) in cells.iter_mut().enumerate() {
        for j in 0..FIELD_ELEMENTS_PER_CELL {
            let index = i * FIELD_ELEMENTS_PER_CELL + j;
            let fr_bytes = data_fr[index].to_bytes();
            cell.bytes[j * BYTES_PER_FIELD_ELEMENT..(j + 1) * BYTES_PER_FIELD_ELEMENT]
                .copy_from_slice(&fr_bytes);
        }
    }

    // Compute proofs
    let mut proofs_g1 = vec![TG1::identity(); CELLS_PER_EXT_BLOB];
    compute_fk20_proofs(&mut proofs_g1, &polynomial, FIELD_ELEMENTS_PER_BLOB, s);
    reverse_bit_order(&mut proofs_g1)?;

    Ok((cells, proofs))
}
