use crate::{G1Affine, G1Fp, G1GetFp, G1ProjAddAffine, Scalar256, G1};
use alloc::vec::Vec;

#[cfg(feature = "arkmsm")]
use super::arkmsm::arkmsm_msm::VariableBaseMSM;

#[cfg(not(feature = "arkmsm"))]
use super::tiling_pippenger_ops::tiling_pippenger;

#[cfg(feature = "parallel")]
use super::tiling_parallel_pippenger::{parallel_affine_conv, tiling_parallel_pippenger};

#[cfg(feature = "parallel")]
pub fn msm_parallel<
    TG1: G1 + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    tiling_parallel_pippenger(points, scalars)
}

pub fn msm_sequential<
    TG1: G1 + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    #[cfg(not(feature = "arkmsm"))]
    {
        tiling_pippenger(points, scalars)
    }

    #[cfg(feature = "arkmsm")]
    {
        VariableBaseMSM::multi_scalar_mul::<TG1, TG1Fp, TG1Affine, TProjAddAffine>(points, scalars)
    }
}

pub fn msm<
    TG1: G1 + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    #[cfg(feature = "parallel")]
    return msm_parallel::<TG1, TG1Fp, TG1Affine, TProjAddAffine>(points, scalars);

    #[cfg(not(feature = "parallel"))]
    return msm_sequential::<TG1, TG1Fp, TG1Affine, TProjAddAffine>(points, scalars);
}

pub fn batch_convert<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp> + Sized>(
    points: &[TG1],
) -> Vec<TG1Affine> {
    #[cfg(feature = "parallel")]
    return parallel_affine_conv::<TG1, TFp, TG1Affine>(points);

    #[cfg(not(feature = "parallel"))]
    return TG1Affine::into_affines(points);
}
