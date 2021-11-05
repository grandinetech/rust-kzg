use super::kzg_proofs::{fr_add, /*fr_one, fr_zero,*/ FFTSettings};
use super::utils::{
    blst_fr_into_pc_fr, blst_poly_into_pc_poly, pc_fr_into_blst_fr, pc_poly_into_blst_poly,
    PolyData,
};
use crate::kzg_types::FsFr as BlstFr;
use ark_bls12_381::Fr;
use ark_std::{log2};
use kzg::{Fr as FrTrait,/* Poly, FFTSettings as FFTSettingsT, FFTFr*/};
// use merkle_light::merkle::{log2_pow2, next_pow2};
use std::cmp::{min/*, max*/};
use std::ops::Neg;
// use crate::zero_poly::pad_poly;

// pub(crate) fn from_u64(n: u64) -> BlstFr {
//     pc_fr_into_blst_fr(Fr::from(n))
// }

pub(crate) fn neg(n: BlstFr) -> BlstFr {
    pc_fr_into_blst_fr(blst_fr_into_pc_fr(&n).neg())
}

// pub(crate) fn inverse(n: PolyData, len: usize) -> Result<PolyData, Error> {
//     Ok(n)
// }

// pub(crate) fn is_one(n: BlstFr) -> bool {
//     n.equals(&BlstFr::one())
// }

pub(crate) fn poly_inverse(b: &PolyData, output_len: usize) -> Result<PolyData, String> {
    assert!(b.coeffs.len() > 0);
    assert!(!BlstFr::is_zero(&b.coeffs[0]));

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

    // let scale: usize = log2_pow2(next_pow2(2 * output_len - 1));

    // let fs: FFTSettings = FFTSettings::from_scale(scale).unwrap();

    output.coeffs[0] = b.coeffs[0].inverse();
    let mut d: usize = 0;
    let mut mask: usize = 1 << log2(maxd);

    while mask != 0 {
        d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
        mask = mask >> 1;

        let len_temp: usize = min(d + 1, b.coeffs.len() + output.coeffs.len() - 1);

        let p1 = blst_poly_into_pc_poly(&output).unwrap();
        let p2 = blst_poly_into_pc_poly(b).unwrap();
        let mut tmp0 = pc_poly_into_blst_poly(&p1 * &p2).unwrap();

        for i in 0..len_temp {
            let cloned_fr = tmp0.coeffs[i].clone();
            tmp0.coeffs[i] = neg(cloned_fr);
        }
        let fr_two = pc_fr_into_blst_fr(Fr::from(2));
        tmp0.coeffs[0] = fr_add(&tmp0.coeffs[0], &fr_two);

        let p1 = blst_poly_into_pc_poly(&tmp0).unwrap();
        let p2 = blst_poly_into_pc_poly(&output).unwrap();
        let mut tmp1 = pc_poly_into_blst_poly(&p1 * &p2).unwrap();
        // println!("tmp0 {:?}", tmp0.coeffs.len());
        // println!("tmp1 {:?}", tmp1.coeffs.len());
        // println!("output {:?}", output.coeffs.len());
        // println!("len_temp {:?}", len_temp);
        // println!("output_len {:?}", output_len);

        if tmp1.coeffs.len() > output_len {
            tmp1.coeffs = tmp1.coeffs[..output_len].to_vec();
        }
        for i in 0..tmp1.coeffs.len() {
            output.coeffs[i] = tmp1.coeffs[i];
        }
    }
    assert!(d + 1 == output_len);
    Ok(output)
}

// pub(crate) fn rand_fr() -> BlstFr {
//     let rng = &mut test_rng();
//     pc_fr_into_blst_fr(Fr::rand(rng))
// }


pub(crate) fn poly_mul_fft(p1: &PolyData, p2: &PolyData, _fs: Option<&FFTSettings>, _len: usize) -> Result<PolyData, String> {
    Ok(pc_poly_into_blst_poly(
        &blst_poly_into_pc_poly(p1).unwrap() * &blst_poly_into_pc_poly(p2).unwrap(),
    )
    .unwrap())
}

pub(crate) fn poly_mul_direct(p1: &PolyData, p2: &PolyData, _len: usize) -> Result<PolyData, String> {
    Ok(pc_poly_into_blst_poly(
        blst_poly_into_pc_poly(p1).unwrap().naive_mul(&blst_poly_into_pc_poly(p2).unwrap())
    )
    .unwrap())
}

pub(crate) fn poly_long_div(p1: &PolyData, p2: &PolyData) -> PolyData {
    pc_poly_into_blst_poly(
        &blst_poly_into_pc_poly(p1).unwrap() / &blst_poly_into_pc_poly(p2).unwrap(),
    )
    .unwrap()
}

pub(crate) fn poly_fast_div(p1: &PolyData, p2: &PolyData) -> Result<PolyData, String> {
    pc_poly_into_blst_poly(
        &blst_poly_into_pc_poly(p1).unwrap() / &blst_poly_into_pc_poly(p2).unwrap(),
    )
}

pub fn poly_mul(a: &PolyData, b: &PolyData, fs: FFTSettings, len: usize)  -> Result<PolyData, String> {
    if a.coeffs.len() < 64 || b.coeffs.len() < 64 || len < 128 {
        poly_mul_direct(a, b, len)
    } else {
        poly_mul_fft(a, b, Some(&fs), len)
    }
}

// pub(crate) fn poly_mul_direct(a: &PolyData, b: &PolyData, len: usize) -> Result<PolyData, String> {
//     //     uint64_t a_degree = a->length - 1;
//     // uint64_t b_degree = b->length - 1;

//     let a_degree = a.coeffs.len() - 1;
//     let b_degree = b.coeffs.len() - 1;
//     let mut out = PolyData::new(len).unwrap();

//     for k in 0..len {
//         out.coeffs[k] = BlstFr::zero();
//     }

//     // Truncate the output to the length of the output polynomial
//     for i in 0..a_degree{
//         // for (uint64_t j = 0; j <= b_degree && i + j < out->length; j++) {
//         //     fr_t tmp;
//         //     fr_mul(&tmp, &a->coeffs[i], &b->coeffs[j]);
//         //     fr_add(&out->coeffs[i + j], &out->coeffs[i + j], &tmp);
//         // }
//         let mut j: usize = 0;
//             while j <= b_degree && i + j < len {
//                 let tmp = a.coeffs[i].mul(&b.coeffs[j]);
//                 out.coeffs[i+j] = out.coeffs[i+j].add(&tmp);
//                 j += 1;
//             }
//     }

//     Ok(out)
// }


// pub fn poly_mul_fft(a: &PolyData, b: &PolyData, fs: Option<&FFTSettings>, len: usize) -> Result<PolyData, String>  {
//     // Truncate a and b so as not to do excess work for the number of coefficients required.
//     // uint64_t a_len = min_u64(a->length, out->length);
//     // uint64_t b_len = min_u64(b->length, out->length);
//     // uint64_t length = next_power_of_two(a_len + b_len - 1);

//     let a_len = min(a.len(), len);
//     let b_len = min(b.len(), len);
//     let length = (a_len + b_len - 1).next_power_of_two();

//     // If the FFT settings are NULL then make a local set, otherwise use the ones passed in.
//     let mut fs_p = FFTSettings::default();
//     if !fs.is_none() {
//         fs_p = fs.unwrap().clone();
//     } else {
//         let scale = log2_pow2(length); // TODO only good up to length < 32 bits
//         fs_p = FFTSettings::new(scale).unwrap();
//     }
//     assert!(length <= fs_p.max_width);

//     // fr_t *a_pad, *b_pad, *a_fft, *b_fft;
//     // TRY(new_fr_array(&a_pad, length));
//     // TRY(new_fr_array(&b_pad, length));

//     let mut a_pad = PolyData::new(length).unwrap();
//     let mut b_pad = PolyData::new(length).unwrap();
//     a_pad.coeffs = pad_poly(a, length).unwrap();
//     b_pad.coeffs = pad_poly(b, length).unwrap();

//     // TRY(new_fr_array(&a_fft, length));
//     // TRY(new_fr_array(&b_fft, length));
//     // TRY(fft_fr(a_fft, a_pad, false, length, fs_p));
//     // TRY(fft_fr(b_fft, b_pad, false, length, fs_p));

//     let a_fft = fs_p.fft_fr(&a_pad.coeffs, false).unwrap();
//     let b_fft = fs_p.fft_fr(&b_pad.coeffs, false).unwrap();

//     // fr_t *ab_fft = a_pad; // reuse the a_pad array
//     // fr_t *ab = b_pad;     // reuse the b_pad array

//     let mut ab_fft = a_pad;
//     let mut ab = b_pad;

//     for i in 0..length {
//         // fr_mul(&ab_fft[i], &a_fft[i], &b_fft[i]);
//         ab_fft.coeffs[i] = a_fft[i].mul(&b_fft[i]);
//     }
//     // TRY(fft_fr(ab, ab_fft, true, length, fs_p));

//     ab.coeffs = fs_p.fft_fr(&ab_fft.coeffs, true).unwrap();

//     // Copy result to output
//     // uint64_t data_len = min_u64(out->length, length);

//     let data_len = min(len, length);

//     let mut out = PolyData::new(max(len, length)).unwrap();

//     for i in 0..data_len {
//         out.coeffs[i] = ab.coeffs[i];
//     }
//     for i in data_len..len {
//         out.coeffs[i] = BlstFr::zero();
//     }

//     Ok(out)
// }

// pub fn poly_fast_div(dividend: &PolyData, divisor: &PolyData) -> Result<PolyData, String> {

//     assert!(divisor.coeffs.len() > 0);


//     println!("DIVICOS: {:?}", divisor);
//     println!("ZERO: {:?}", BlstFr::zero());
//     assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

//     let m: usize = dividend.coeffs.len() - 1;
//     let n: usize = divisor.coeffs.len() - 1;

//     if n > m {
//         return Ok(PolyData { coeffs: Vec::default() });
//     }


//     assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

//     let mut out = PolyData { coeffs: Vec::default() };

//     if divisor.coeffs.len() == 1 {

//         for i in 0..dividend.coeffs.len() {
//             out.coeffs.push(fr_div(&dividend.coeffs[i], &divisor.coeffs[0]).unwrap());
//         }
//         return Ok(out);
//     }


//     let mut a_flip = PolyData { coeffs: Vec::default() };
//     let mut b_flip = PolyData { coeffs: Vec::default() };


//     a_flip = poly_flip(&dividend).unwrap();
//     b_flip = poly_flip(&divisor).unwrap();

//     // poly inv_b_flip;
//     let mut inv_b_flip = PolyData { coeffs: Vec::default() };

//     inv_b_flip = poly_inverse(&b_flip, m - n + 1).unwrap();

//     let mut q_flip = PolyData { coeffs: Vec::default() };


//     q_flip = a_flip.mul_direct(&inv_b_flip, m - n + 1).unwrap();


//     out = poly_flip(&q_flip).unwrap();

//     Ok(out)
// }

// pub fn fr_div(a: &BlstFr, b: &BlstFr) -> Result<BlstFr, String> {
        // let tmp = b.eucl_inverse();
        // let out = a.mul(&tmp);
//     Ok(BlstFr(out))
// }

// pub fn poly_flip(input: &PolyData) -> Result<PolyData, String> {
//     let mut output = PolyData { coeffs: Vec::default() };
//     for i in 0..input.coeffs.len() {
//         output.coeffs.push(input.coeffs[input.coeffs.len() - i - 1]);
//     }
//     Ok(output)
// }