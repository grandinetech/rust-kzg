use crate::{
    common_utils::log2_u64, msm::bucket_msm::BucketMSM, msm::glv::decompose,
    msm::types::G1_SCALAR_SIZE_GLV, Fr, G1Affine, G1Fp, G1ProjAddAffine, Scalar256, G1,
};

use super::types::G1_SCALAR_SIZE;

pub struct VariableBaseMSM;

impl VariableBaseMSM {
    /// WARNING: this function is derived from benchmark results running
    /// on a Ubuntu 20.04.2 LTS server with AMD EPYC 7282 16-Core CPU
    /// and 128G memory, the optimal performance may vary on a different
    /// configuration.
    const fn get_opt_window_size(k: u32) -> u32 {
        match k {
            0..=9 => 8,
            10..=12 => 10,
            13..=14 => 12,
            15..=19 => 13,
            20..=22 => 15,
            23.. => 16,
        }
    }

    pub fn msm_slice(mut scalar: Scalar256, slices: &mut [u32], window_bits: u32) {
        assert!(window_bits <= 31); // reserve one bit for marking signed slices

        let mut carry = 0;
        let total = 1 << window_bits;
        let half = total >> 1;
        slices.iter_mut().for_each(|el| {
            *el = (scalar.data.as_ref()[0] % (1 << window_bits)) as u32;
            scalar.divn(window_bits);

            *el += carry;
            if half < *el {
                // slices[i] == half is okay, since (slice[i]-1) will be used for bucket_id
                *el = total - *el;
                carry = 1;
                *el |= 1 << 31; // mark the highest bit for later
            } else {
                carry = 0;
            }
        });
        assert!(
            carry == 0,
            "msm_slice overflows when apply signed-bucket-index"
        );
    }

    #[allow(dead_code)]
    fn multi_scalar_mul_g1_glv<
        TG1: G1,
        TG1Fp: G1Fp,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
        TFr: Fr,
    >(
        points: &[TG1Affine],
        scalars: &[Scalar256],
        window_bits: u32,
        max_batch: u32,
        max_collisions: u32,
    ) -> TG1 {
        let num_slices: usize = ((G1_SCALAR_SIZE_GLV + window_bits - 1) / window_bits) as usize;
        let mut bucket_msm = BucketMSM::<TG1, TG1Fp, TG1Affine, TProjAddAffine>::new(
            G1_SCALAR_SIZE_GLV,
            window_bits,
            max_batch,
            max_collisions,
        );
        // scalar = phi * lambda + normal
        let mut phi_slices = vec![0u32; num_slices];
        let mut normal_slices = vec![0u32; num_slices];

        scalars
            .iter()
            .zip(points)
            .filter(|(s, _)| !s.is_zero())
            .for_each(|(scalar, point)| {
                let (phi, _normal, is_neg_scalar, is_neg_normal) =
                    decompose(&TFr::from_u64_arr(&scalar.data), window_bits);

                Self::msm_slice(
                    Scalar256::from_u64(phi.to_u64_arr()),
                    &mut phi_slices[..num_slices],
                    window_bits,
                );
                Self::msm_slice(
                    Scalar256::from_u64(phi.to_u64_arr()),
                    &mut normal_slices[..num_slices],
                    window_bits,
                );
                bucket_msm.process_point_and_slices_glv(
                    point,
                    &normal_slices[..num_slices],
                    &phi_slices[..num_slices],
                    is_neg_scalar,
                    is_neg_normal,
                );
            });

        bucket_msm.process_complete();
        bucket_msm.batch_reduce()
    }

    fn multi_scalar_mul_general<
        TG1: G1,
        TG1Fp: G1Fp,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    >(
        points: &[TG1Affine],
        scalars: &[Scalar256],
        window_bits: u32,
        max_batch: u32,
        max_collisions: u32,
    ) -> TG1 {
        let num_slices: usize = ((G1_SCALAR_SIZE + window_bits - 1) / window_bits) as usize;
        let mut bucket_msm = BucketMSM::<TG1, TG1Fp, TG1Affine, TProjAddAffine>::new(
            G1_SCALAR_SIZE,
            window_bits,
            max_batch,
            max_collisions,
        );

        let mut slices = vec![0u32; num_slices];
        scalars
            .iter()
            .zip(points)
            .filter(|(s, _)| !s.is_zero())
            .for_each(|(&scalar, point)| {
                Self::msm_slice(scalar, &mut slices[..num_slices], window_bits);
                bucket_msm.process_point_and_slices(point, &slices[..num_slices]);
            });

        bucket_msm.process_complete();
        bucket_msm.batch_reduce()
    }

    pub fn multi_scalar_mul<
        TG1: G1,
        TG1Fp: G1Fp,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    >(
        points: &[TG1Affine],
        scalars: &[Scalar256],
    ) -> TG1 {
        let opt_window_size = Self::get_opt_window_size(log2_u64(points.len()) as u32);
        // Self::multi_scalar_mul_g1_glv::<TG1, TG1Fp, TG1Affine, TProjAddAffine, TFr>(points, scalars, opt_window_size, 2048, 256)
        Self::multi_scalar_mul_general::<TG1, TG1Fp, TG1Affine, TProjAddAffine>(
            points,
            scalars,
            opt_window_size,
            2048,
            256,
        )
    }
}
