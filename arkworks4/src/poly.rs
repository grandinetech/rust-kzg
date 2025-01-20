use super::kzg_proofs::FFTSettings;
use super::utils::{blst_poly_into_pc_poly, PolyData};
use crate::kzg_types::ArkFr as BlstFr;
use crate::utils::pc_poly_into_blst_poly;
use crate::zero_poly::pad_poly;
use ark_bls12_381::Fr;
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_std::{log2, Zero};
use kzg::common_utils::{log2_pow2, next_pow_of_2};
use kzg::{FFTFr, FFTSettings as FFTSettingsT, Fr as FrTrait, Poly};
use std::cmp::min;

pub fn poly_inverse(b: &PolyData, output_len: usize) -> Result<PolyData, String> {
    if b.coeffs.is_empty() {
        return Err(String::from("b.coeffs is empty"));
    }

    if BlstFr::is_zero(&b.coeffs[0]) {
        return Err(String::from("b.coeffs[0] is zero"));
    }

    let mut output = PolyData {
        coeffs: vec![BlstFr::zero(); output_len],
    };
    if b.coeffs.len() == 1 {
        output.coeffs[0] = b.coeffs[0].inverse();
        for i in 1..output_len {
            output.coeffs[i] = BlstFr::zero();
        }
        return Ok(output);
    }

    let maxd = output_len - 1;
    let scale = next_pow_of_2(log2_pow2(2 * output_len - 1));
    let fs = FFTSettings::new(scale).unwrap();

    let mut tmp0: PolyData;
    let mut tmp1: PolyData;

    output.coeffs[0] = b.coeffs[0].inverse();
    let mut d: usize = 0;
    let mut mask: usize = 1 << log2(maxd);

    while mask != 0 {
        d = 2 * d + usize::from((maxd & mask) != 0);
        mask >>= 1;

        let len_temp: usize = min(d + 1, b.coeffs.len() + output.coeffs.len() - 1);

        tmp0 = poly_mul(b, &output, Some(&fs), len_temp).unwrap();

        for i in 0..len_temp {
            tmp0.coeffs[i] = tmp0.coeffs[i].negate();
        }
        let fr_two = BlstFr { fr: Fr::from(2) };
        tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

        let len_temp2: usize = d + 1;

        tmp1 = poly_mul(&output, &tmp0, Some(&fs), len_temp2).unwrap();

        if tmp1.coeffs.len() > output_len {
            tmp1.coeffs = tmp1.coeffs[..output_len].to_vec();
        }
        for i in 0..tmp1.coeffs.len() {
            output.coeffs[i] = tmp1.coeffs[i];
        }
    }
    if d + 1 != output_len {
        return Err(String::from("d + 1 is not equals to output_len"));
    }
    Ok(output)
}

pub fn poly_mul_direct(p1: &PolyData, p2: &PolyData, len: usize) -> Result<PolyData, String> {
    let p1 = blst_poly_into_pc_poly(&p1.coeffs);
    let p2 = blst_poly_into_pc_poly(&p2.coeffs);
    if p1.is_zero() || p2.is_zero() {
        Ok(pc_poly_into_blst_poly(DensePolynomial::zero()))
    } else {
        let mut result = vec![Fr::zero(); len];
        for (i, self_coeff) in p1.coeffs.iter().enumerate() {
            for (j, other_coeff) in p2.coeffs.iter().enumerate() {
                if i + j >= len {
                    break;
                }
                result[i + j] += &(*self_coeff * other_coeff);
            }
        }
        let p = pc_poly_into_blst_poly(DensePolynomial::from_coefficients_vec(result));
        Ok(PolyData {
            coeffs: pad_poly(&p, len).unwrap(),
        })
    }
}

pub fn poly_long_div(p1: &PolyData, p2: &PolyData) -> Result<PolyData, String> {
    Ok(pc_poly_into_blst_poly(
        &blst_poly_into_pc_poly(&p1.coeffs) / &blst_poly_into_pc_poly(&p2.coeffs),
    ))
}

pub fn poly_mul(
    a: &PolyData,
    b: &PolyData,
    fs: Option<&FFTSettings>,
    len: usize,
) -> Result<PolyData, String> {
    if a.coeffs.len() < 64 || b.coeffs.len() < 64 || len < 128 {
        poly_mul_direct(a, b, len)
    } else {
        poly_mul_fft(a, b, fs, len)
    }
}

pub fn poly_mul_fft(
    a: &PolyData,
    b: &PolyData,
    fs: Option<&FFTSettings>,
    len: usize,
) -> Result<PolyData, String> {
    // Truncate a and b so as not to do excess work for the number of coefficients required.
    let a_len = min(a.len(), len);
    let b_len = min(b.len(), len);
    let length = next_pow_of_2(a_len + b_len - 1);

    // If the FFT settings are NULL then make a local set, otherwise use the ones passed in.
    let fs_p = if let Some(x) = fs {
        x.clone()
    } else {
        let scale = log2_pow2(length);
        FFTSettings::new(scale).unwrap()
    };

    if length > fs_p.max_width {
        return Err(String::from(
            "length should be equals or less than FFTSettings max width",
        ));
    }

    let a = PolyData {
        coeffs: a.coeffs[..a_len].to_vec(),
    };
    let b = PolyData {
        coeffs: b.coeffs[..b_len].to_vec(),
    };
    let a_pad = PolyData {
        coeffs: pad_poly(&a, length).unwrap(),
    };
    let b_pad = PolyData {
        coeffs: pad_poly(&b, length).unwrap(),
    };

    let a_fft;
    let b_fft;
    #[cfg(feature = "parallel")]
    {
        if length > 1024 {
            let mut a_fft_temp = vec![];
            let mut b_fft_temp = vec![];

            rayon::join(
                || a_fft_temp = fs_p.fft_fr(&a_pad.coeffs, false).unwrap(),
                || b_fft_temp = fs_p.fft_fr(&b_pad.coeffs, false).unwrap(),
            );

            a_fft = a_fft_temp;
            b_fft = b_fft_temp;
        } else {
            a_fft = fs_p.fft_fr(&a_pad.coeffs, false).unwrap();
            b_fft = fs_p.fft_fr(&b_pad.coeffs, false).unwrap();
        }
    }
    #[cfg(not(feature = "parallel"))]
    {
        a_fft = fs_p.fft_fr(&a_pad.coeffs, false).unwrap();
        b_fft = fs_p.fft_fr(&b_pad.coeffs, false).unwrap();
    }
    let mut ab_fft = a_pad;
    let mut ab = b_pad;

    for i in 0..length {
        ab_fft.coeffs[i] = a_fft[i].mul(&b_fft[i]);
    }

    ab.coeffs = fs_p.fft_fr(&ab_fft.coeffs, true).unwrap();

    let data_len = min(len, length);
    let mut out = PolyData::new(len);

    for i in 0..data_len {
        out.coeffs[i] = ab.coeffs[i];
    }
    for i in data_len..len {
        out.coeffs[i] = BlstFr::zero();
    }

    Ok(out)
}

pub fn poly_fast_div(dividend: &PolyData, divisor: &PolyData) -> Result<PolyData, String> {
    if divisor.coeffs.is_empty() {
        return Err(String::from("divisor coeffs are empty"));
    }

    if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
        return Err(String::from("divisor coeffs last member is zero"));
    }

    let m = dividend.coeffs.len() - 1;
    let n = divisor.coeffs.len() - 1;

    if n > m {
        return Ok(PolyData::new(0));
    }

    if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
        return Err(String::from("divisor coeffs last member is zero"));
    }

    let mut out = PolyData::new(0);

    if divisor.len() == 1 {
        for i in 0..dividend.len() {
            out.coeffs
                .push(dividend.coeffs[i].div(&divisor.coeffs[0]).unwrap());
        }
        return Ok(out);
    }

    let a_flip = poly_flip(dividend).unwrap();
    let b_flip = poly_flip(divisor).unwrap();

    let inv_b_flip = poly_inverse(&b_flip, m - n + 1).unwrap();
    let q_flip = poly_mul(&a_flip, &inv_b_flip, None, m - n + 1).unwrap();

    out = poly_flip(&q_flip).unwrap();

    Ok(PolyData {
        coeffs: out.coeffs[..m - n + 1].to_vec(),
    })
}

pub fn poly_flip(input: &PolyData) -> Result<PolyData, String> {
    let mut output = PolyData::new(0);
    for i in 0..input.len() {
        output.coeffs.push(input.coeffs[input.coeffs.len() - i - 1]);
    }
    Ok(output)
}
