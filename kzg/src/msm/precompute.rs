extern crate alloc;

use alloc::string::String;

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1};

#[cfg(all(feature = "bgmw", any(not(feature = "arkmsm"), feature = "parallel")))]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine> =
    super::bgmw::BgmwTable<TFr, TG1, TG1Fp, TG1Affine>;

#[cfg(any(
    not(feature = "bgmw"),
    all(feature = "arkmsm", not(feature = "parallel"))
))]
#[derive(Debug, Clone)]
pub struct EmptyTable<TFr, TG1, TG1Fp, TG1Affine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
{
    fr_marker: core::marker::PhantomData<TFr>,
    g1_marker: core::marker::PhantomData<TG1>,
    g1_fp_marker: core::marker::PhantomData<TG1Fp>,
    g1_affine_marker: core::marker::PhantomData<TG1Affine>,
}

#[cfg(any(
    not(feature = "bgmw"),
    all(feature = "arkmsm", not(feature = "parallel"))
))]
impl<TFr, TG1, TG1Fp, TG1Affine> EmptyTable<TFr, TG1, TG1Fp, TG1Affine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
{
    fn new(_: &[TG1]) -> Result<Option<Self>, String> {
        Ok(None)
    }

    pub fn multiply_sequential(&self, _: &[crate::Scalar256]) -> TG1 {
        panic!("This function must not be called")
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, _: &[crate::Scalar256]) -> TG1 {
        panic!("This function must not be called")
    }
}

#[cfg(any(
    not(feature = "bgmw"),
    all(feature = "arkmsm", not(feature = "parallel"))
))]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine> = EmptyTable<TFr, TG1, TG1Fp, TG1Affine>;

pub fn precompute<TFr, TG1, TG1Fp, TG1Affine>(
    points: &[TG1],
) -> Result<Option<PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine>>, String>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
{
    PrecomputationTable::<TFr, TG1, TG1Fp, TG1Affine>::new(points)
}
