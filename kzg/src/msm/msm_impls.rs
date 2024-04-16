use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, Scalar256, G1};
use alloc::vec::Vec;

#[cfg(all(feature = "arkmsm", not(feature = "parallel")))]
use super::arkmsm::arkmsm_msm::VariableBaseMSM;
use super::precompute::PrecomputationTable;

#[cfg(all(not(feature = "arkmsm"), not(feature = "parallel")))]
use super::tiling_pippenger_ops::tiling_pippenger;

#[cfg(feature = "parallel")]
use super::tiling_parallel_pippenger::{parallel_affine_conv, tiling_parallel_pippenger};

#[cfg(feature = "parallel")]
fn msm_parallel<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
    precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine>>,
) -> TG1 {
    if let Some(precomputation) = precomputation {
        precomputation.multiply_parallel(scalars)
    } else {
        tiling_parallel_pippenger(points, scalars)
    }
}

#[cfg(not(feature = "parallel"))]
#[allow(clippy::extra_unused_type_parameters)]
#[allow(unused_variables)]
fn msm_sequential<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
    precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine>>,
) -> TG1 {
    #[cfg(not(feature = "arkmsm"))]
    {
        assert!(core::cmp::min(points.len(), scalars.len()) > 1);
        if let Some(precomputation) = precomputation {
            precomputation.multiply_sequential(scalars)
        } else {
            tiling_pippenger(points, scalars)
        }
    }

    #[cfg(feature = "arkmsm")]
    {
        VariableBaseMSM::multi_scalar_mul::<TG1, TG1Fp, TG1Affine, TProjAddAffine>(points, scalars)
    }
}

pub fn batch_convert<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp> + Sized>(
    points: &[TG1],
) -> Vec<TG1Affine> {
    #[cfg(feature = "parallel")]
    return parallel_affine_conv::<TG1, TFp, TG1Affine>(points);

    #[cfg(not(feature = "parallel"))]
    return TG1Affine::into_affines(points);
}

#[allow(clippy::extra_unused_type_parameters)]
pub fn msm<
    TG1: G1 + G1GetFp<TG1Fp> + G1Mul<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    TFr: Fr,
>(
    points: &[TG1],
    scalars: &[TFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine>>,
) -> TG1 {
    if len < 8 {
        let mut out = TG1::default();
        for i in 0..len {
            let tmp = points[i].mul(&scalars[i]);
            out.add_or_dbl_assign(&tmp);
        }
        return out;
    }

    let points = batch_convert::<TG1, TG1Fp, TG1Affine>(&points[0..len]);
    let scalars = scalars[0..len]
        .iter()
        .map(TFr::to_scalar)
        .collect::<Vec<_>>();

    #[cfg(feature = "parallel")]
    return msm_parallel::<TFr, TG1, TG1Fp, TG1Affine>(&points, &scalars, precomputation);

    #[cfg(not(feature = "parallel"))]
    return msm_sequential::<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>(
        &points,
        &scalars,
        precomputation,
    );
}
