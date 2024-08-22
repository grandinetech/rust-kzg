use crate::consts::G1_IDENTITY;
use crate::eip_4844::FIELD_ELEMENTS_PER_CELL;
use crate::kzg_types::{ArkG1, ArkFr as BlstFr};
use crate::utils::PolyData;
use crate::{FFTFr, G1Mul, KZGSettings, G1};
use crate::utils::PolyData;
use kzg::{G1, G1Mul, FFTSetings, FFTFr};


fn compute_fk20_proofs<
    TFr: FFTFr, 
    TG1: G1 + G1Mul
>(
    p: &PolyData,
    n: usize,
    s: &KZGSettings
) -> Result<Vec<TG1>, String>{
    let k = n / FIELD_ELEMENTS_PER_CELL;
    let k2 = k * 2;

    let mut toeplitz_coeff = vec![TG1::default(); k2];
    let mut h = vec![TG1::identity(); k2];
    let mut h_ext_fft = vec![TG1::identity(); k2];

    for i in 0..FIELD_ELEMENTS_PER_CELL {
        toeplitz_coeffs_stride(p, &mut toeplitz_coeffs, n, FIELD_ELEMENTS_PER_CELL)?;
        s.get_fft_settings().fft_fr(&toeplitz_coeffs, false)?;
        for j in 0..k2 {
            h_ext_fft[j] = h_ext_fft[j].add_or_dbl(&s.x_ext_fft[j].mul(&toeplitz_coeffs[j]));
        }
    }

    s.get_fft_settings().fft_g1(&h_ext_fft, false)?;

    for i in h.iter_mut().take(k) {
        *i = h_ext_fft[i.len() - 1];
    }
    for i in h.iter_mut().take(k2).skip(k) {
        *i = G1_IDENTITY;
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