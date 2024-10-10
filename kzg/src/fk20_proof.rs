// use crate::consts::G1_IDENTITY;
// use crate::eip_4844::FIELD_ELEMENTS_PER_CELL;
// use crate::kzg_types::{ArkFr as BlstFr, ArkG1};
// use crate::utils::PolyData;
// use crate::utils::PolyData;
// use crate::{FFTFr, G1Mul, KZGSettings, G1};
// use kzg::{FFTFr, FFTSetings, G1Mul, G1};

use crate::{
    eip_4844::FIELD_ELEMENTS_PER_CELL, FFTFr, FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul,
    KZGSettings, Poly, G1, G2,
};

fn compute_fk20_proofs<
    TFr: Fr,
    TFFTFr: FFTFr<TFr>,
    TG1Fp: G1Fp,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TPoly: Poly<TFr>,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
    TFFTSettings: FFTSettings<TFr>,
>(
    p: &TPoly,
    n: usize,
    s: &TKZGSettings,
) -> Result<Vec<TG1>, String> {
    let k = n / FIELD_ELEMENTS_PER_CELL;
    let k2 = k * 2;

    let mut toeplitz_coeff = vec![TG1::default(); k2];
    let mut h = vec![TG1::identity(); k2];
    let mut h_ext_fft = vec![TG1::identity(); k2];

    for i in 0..FIELD_ELEMENTS_PER_CELL {
        toeplitz_coeffs_stride(p, &mut toeplitz_coeff, n, FIELD_ELEMENTS_PER_CELL)?;
        s.get_fft_settings().fft_fr(&toeplitz_coeff, false)?;
        for j in 0..k2 {
            h_ext_fft[j] = h_ext_fft[j].add_or_dbl(&s.x_ext_fft[j].mul(&toeplitz_coeff[j]));
        }
    }

    s.get_fft_settings().fft_g1(&h_ext_fft, false)?;

    for i in h.iter_mut().take(k) {
        *i = h_ext_fft[i.len() - 1];
    }
    for i in h.iter_mut().take(k2).skip(k) {
        *i = TG1::identity();
    }

    s.get_fft_settings().fft_g1(h.as_mut_slice(), false)?;

    Ok(h)
}

fn toeplitz_coeffs_stride(
    poly: &PolyData,
    offset: usize,
    stride: usize,
    outlen: usize,
) -> Result<PolyData, String> {
    let n = poly.len();

    if stride == 0 {
        return Err(String::from("stride must be greater than 0"));
    }

    let k = n / stride;
    let k2 = k * 2;

    if outlen < k2 {
        return Err(String::from("outlen must be equal or greater than k2"));
    }

    let mut out = PolyData::new(outlen);
    out.set_coeff_at(0, &poly.coeffs[n - 1 - offset]);
    let mut i = 1;
    while i <= (k + 1) && i < k2 {
        out.set_coeff_at(i, &BlstFr::zero());
        i += 1;
    }
    let mut j = 2 * stride - offset - 1;
    for i in (k + 2)..k2 {
        out.set_coeff_at(i, &poly.coeffs[j]);
        j += stride;
    }
    Ok(out)
}
