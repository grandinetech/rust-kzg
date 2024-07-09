use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine};

use kzg::cfg_into_iter;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use ark_ec::ProjectiveCurve;
use ark_ff::PrimeField;
use kzg::msm::precompute::PrecomputationTable;
use kzg::{Fr as KzgFr, G1Mul};
use kzg::{FFTG1, G1};
use std::ops::MulAssign;

extern crate alloc;

#[allow(unused_variables)]
pub fn g1_linear_combination(
    out: &mut ArkG1,
    points: &[ArkG1],
    scalars: &[ArkFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<ArkFr, ArkG1, ArkFp, ArkG1Affine>>,
) {
    #[cfg(feature = "sppark")]
    {
        use ark_bls12_381::{Fr, G1Affine};
        use ark_ec::msm::VariableBaseMSM;
        use ark_ff::BigInteger256;
        use kzg::{G1Mul, G1};

        if len < 8 {
            *out = ArkG1::default();
            for i in 0..len {
                let tmp = points[i].mul(&scalars[i]);
                out.add_or_dbl_assign(&tmp);
            }

            return;
        }

        let scalars =
            unsafe { alloc::slice::from_raw_parts(scalars.as_ptr() as *const BigInteger256, len) };

        let point = if let Some(precomputation) = precomputation {
            rust_kzg_arkworks3_sppark::multi_scalar_mult_prepared::<G1Affine>(
                precomputation.table,
                scalars,
            )
        } else {
            let affines = kzg::msm::msm_impls::batch_convert::<ArkG1, ArkFp, ArkG1Affine>(&points);
            let affines =
                unsafe { alloc::slice::from_raw_parts(affines.as_ptr() as *const G1Affine, len) };
            rust_kzg_arkworks3_sppark::multi_scalar_mult::<G1Affine>(&affines[0..len], scalars)
        };

        *out = ArkG1(point);
    }

    #[cfg(feature = "sppark_wlc")]
    {
        use ark_bls12_381::{Fr, G1Affine};
        use ark_ff::BigInteger256;
        use kzg::{G1Mul, G1};
        use rust_kzg_arkworks3_sppark_wlc::MultiScalarMultContext;

        if len < 8 {
            *out = ArkG1::default();
            for i in 0..len {
                let tmp = points[i].mul(&scalars[i]);
                out.add_or_dbl_assign(&tmp);
            }

            return;
        }

        let ark_scalars = cfg_into_iter!(&scalars[0..len])
            .map(|scalar| scalar.fr.into_repr())
            .collect::<Vec<_>>();

        let mut context = if let Some(context) = precomputation {
            let table = context.table;
            MultiScalarMultContext { context: table }
        } else {
            let affines = kzg::msm::msm_impls::batch_convert::<ArkG1, ArkFp, ArkG1Affine>(&points);
            let affines =
                unsafe { alloc::slice::from_raw_parts(affines.as_ptr() as *const G1Affine, len) };

            rust_kzg_arkworks3_sppark_wlc::multi_scalar_mult_init(affines)
        };

        let msm_results = rust_kzg_arkworks3_sppark_wlc::multi_scalar_mult::<G1Affine>(
            &mut context,
            len,
            unsafe { std::mem::transmute::<&[_], &[BigInteger256]>(&ark_scalars) },
        );

        *out = ArkG1(msm_results[0]);

        if precomputation.is_none() {
            rust_kzg_arkworks3_sppark_wlc::multi_scalar_mult_free(&mut context);
        }
    }

    #[cfg(not(any(feature = "sppark", feature = "sppark_wlc")))]
    {
        use ark_ec::msm::VariableBaseMSM;
        let ark_points = cfg_into_iter!(&points[0..len])
            .map(|point| point.0.into_affine())
            .collect::<Vec<_>>();
        let ark_scalars = cfg_into_iter!(&scalars[0..len])
            .map(|scalar| scalar.fr.into_repr())
            .collect::<Vec<_>>();
        *out = ArkG1(VariableBaseMSM::multi_scalar_mul(
            ark_points.as_slice(),
            ark_scalars.as_slice(),
        ));
    }
}

pub fn make_data(data: usize) -> Vec<ArkG1> {
    let mut vec = Vec::new();
    if data != 0 {
        vec.push(ArkG1::generator());
        for i in 1..data as u64 {
            let res = vec[(i - 1) as usize].add_or_dbl(&ArkG1::generator());
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
            let inv_fr_len = ArkFr::from_u64(data.len() as u64).inverse();
            ret[..data.len()]
                .iter_mut()
                .for_each(|f| f.0.mul_assign(inv_fr_len.fr));
        }
        Ok(ret)
    }
}

pub fn fft_g1_slow(
    ret: &mut [ArkG1],
    data: &[ArkG1],
    stride: usize,
    roots: &[ArkFr],
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
    roots: &[ArkFr],
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
                1,
            );
            fft_g1_fast(
                &mut ret[half..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
                1,
            );
        }

        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add_or_dbl(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}
