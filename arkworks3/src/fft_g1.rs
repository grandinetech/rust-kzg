use crate::kzg_proofs::LFFTSettings;
use crate::kzg_types::ArkG1ProjAddAffine;
use crate::kzg_types::{ArkFp, ArkG1Affine};
use crate::kzg_types::{ArkFr, ArkG1};

use kzg::msm::msm_impls::msm;

use kzg::msm::precompute::PrecomputationTable;
use kzg::{Fr as KzgFr, G1Mul};
use kzg::{FFTG1, G1};

extern crate alloc;

pub fn g1_linear_combination(
    out: &mut ArkG1,
    points: &[ArkG1],
    scalars: &[ArkFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<ArkFr, ArkG1, ArkFp, ArkG1Affine>>,
) {
    #[cfg(feature = "sppark")]
    {
        use blst::{blst_fr, blst_scalar, blst_scalar_from_fr};
        use kzg::{G1Mul, G1};

        if len < 8 {
            *out = FsG1::default();
            for i in 0..len {
                let tmp = points[i].mul(&scalars[i]);
                out.add_or_dbl_assign(&tmp);
            }

            return;
        }

        let scalars =
            unsafe { alloc::slice::from_raw_parts(scalars.as_ptr() as *const blst_fr, len) };

        let point = if let Some(precomputation) = precomputation {
            rust_kzg_blst_sppark::multi_scalar_mult_prepared(precomputation.table, scalars)
        } else {
            let affines = kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(&points);
            let affines = unsafe {
                alloc::slice::from_raw_parts(affines.as_ptr() as *const blst_p1_affine, len)
            };
            rust_kzg_blst_sppark::multi_scalar_mult(&affines[0..len], &scalars)
        };

        *out = FsG1(point);
    }

    #[cfg(not(feature = "sppark"))]
    {
        *out = msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(
            points,
            scalars,
            len,
            precomputation,
        );
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

impl FFTG1<ArkG1> for LFFTSettings {
    fn fft_g1(&self, data: &[ArkG1], inverse: bool) -> Result<Vec<ArkG1>, String> {
        if data.len() > self.max_width {
            return Err(String::from(
                "Supplied list is longer than the available max width",
            ));
        } else if !data.len().is_power_of_two() {
            return Err(String::from("A list with power-of-two length expected"));
        }

        let stride = self.max_width / data.len();
        let mut ret = vec![ArkG1::default(); data.len()];

        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.roots_of_unity
        };

        fft_g1_fast(&mut ret, data, 1, roots, stride, 1);

        if inverse {
            let inv_fr_len = ArkFr::from_u64(data.len() as u64).inverse();
            ret[..data.len()]
                .iter_mut()
                .for_each(|f| *f = f.mul(&inv_fr_len));
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
        // Evaluate first member at 1
        ret[i] = data[0].mul(&roots[0]);

        // Evaluate the rest of members using a step of (i * J) % data.len() over the roots
        // This distributes the roots over correct x^n members and saves on multiplication
        for j in 1..data.len() {
            let v = data[j * stride].mul(&roots[((i * j) % data.len()) * roots_stride]);
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
                || fft_g1_fast(lo, data, stride * 2, roots, roots_stride * 2, 1),
                || fft_g1_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2, 1),
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
