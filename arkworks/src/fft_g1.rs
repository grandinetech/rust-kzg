use std::ops::{MulAssign, AddAssign, Sub};

use crate::consts::G1_GENERATOR;
use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::{ArkG1, ArkFr as BlstFr};
use crate::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective,
};
use ark_bls12_381::{G1Affine, G1Projective, Fr};
use ark_ec::VariableBaseMSM;

use ark_ec::scalar_mul::fixed_base::FixedBase;
use ark_ec::scalar_mul::variable_base::ChunkedPippenger;
use ark_ff::BigInteger256;
use kzg::{cfg_into_iter, G1Mul, Fr as KzgFr};
use kzg::{FFTG1, G1};
use ark_ec::CurveGroup;
use ark_poly::{EvaluationDomain, DenseUVPolynomial};
// use rust_kzg_blst::

#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;
#[cfg(feature = "parallel")]
use rayon::iter::ParallelIterator;

pub fn g1_linear_combination(out: &mut ArkG1, points: &[ArkG1], scalars: &[BlstFr], len: usize) {
    let ark_points: Vec<G1Affine> = {
        cfg_into_iter!(points)
            .map(|point| {
                point.proj.into_affine()
            })
            .collect()
    };

    let ark_scalars: Vec<BigInteger256> = {
        cfg_into_iter!(scalars)
            .map(|scalar| BigInteger256::from(scalar.fr))
            .collect()
    };
    out.proj = VariableBaseMSM::msm_bigint(ark_points.as_slice(), ark_scalars.as_slice());
}

pub fn make_data(data: usize) -> Vec<ArkG1> {
    let mut vec = Vec::new();
    if data != 0 {
        vec.push(G1_GENERATOR);
        for i in 1..data as u64 {
            let res = vec[(i - 1) as usize].add_or_dbl(&G1_GENERATOR);
            vec.push(res);
        }
    }
    vec
}

impl FFTG1<ArkG1> for FFTSettings {
    fn fft_g1(&self, data: &[ArkG1], inverse: bool) -> Result<Vec<ArkG1>, String> {
        if data.len() > self.max_width {
            return Err(String::from("data length is longer than allowed max width"));
        }
        if !data.len().is_power_of_two() {
            return Err(String::from("data length is not power of 2"));
        }

        let stride: usize = self.max_width / data.len();
        let mut ret = vec![ArkG1::default(); data.len()];

        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };

        fft_g1_fast(&mut ret, data, 1, roots, stride, 1);

        if inverse {
            let inv_fr_len = BlstFr::from_u64(data.len() as u64).inverse();
            ret[..data.len()]
                .iter_mut()
                .for_each(|f| f.proj.mul_assign(&inv_fr_len.fr));
        }
        Ok(ret)
    }
}

pub fn fft_g1_slow(
    ret: &mut [ArkG1],
    data: &[ArkG1],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
    _width: usize,
) {
    for i in 0..data.len() {
        ret[i] = data[0].mul(&roots[0]);
        for j in 1..data.len() {
            let jv = data[j * stride];
            let r = roots[((i * j) % data.len()) * roots_stride];
            let v = jv.mul(&r);
            ret[i] = ret[i].add_or_dbl(&v);
        }
    }
}

pub fn fft_g1_fast(
    ret: &mut [ArkG1],
    data: &[ArkG1],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
    _width: usize,
) {
    let half = ret.len() / 2;
    if half > 0 {
        #[cfg(feature = "parallel")]
        {
            let (lo, hi) = ret.split_at_mut(half);
            rayon::join(
                || fft_g1_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2, 1),
                || fft_g1_fast(lo, data, stride * 2, roots, roots_stride * 2, 1),
            );
        }

        #[cfg(not(feature = "parallel"))]
        {
            fft_g1_fast(
                &mut ret[..half],
                data,
                stride * 2,
                roots,
                roots_stride * 2,
                1
            );
            fft_g1_fast(
                &mut ret[half..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
                1
            );
        }

        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]) ;
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add_or_dbl(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}
