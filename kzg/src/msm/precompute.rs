extern crate alloc;

use alloc::{string::String, vec::Vec};

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, G1};

const _: () = {
    let enabled = cfg!(feature = "arkmsm") as u8
        + cfg!(feature = "bgmw") as u8
        + cfg!(feature = "sppark") as u8
        + cfg!(feature = "wbits") as u8
        + cfg!(feature = "grigaitis_pairwise") as u8
        + cfg!(feature = "grigaitis_pairwise_booth") as u8
        + cfg!(feature = "grigaitis_pairwise_cpdlh") as u8;

    assert!(
        enabled <= 1,
        "incompatible features: select only one of `arkmsm`, `bgmw`, `sppark`, `wbits`, `grigaitis_pairwise`, `grigaitis_pairwise_booth`, `grigaitis_pairwise_cpdlh`"
    );
};

#[cfg(feature = "bgmw")]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    super::bgmw::BgmwTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[cfg(feature = "sppark")]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    super::sppark::SpparkPrecomputation<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[cfg(feature = "wbits")]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    super::wbits::WbitsTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[cfg(feature = "grigaitis_pairwise")]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    super::grigaitis_pairwise::GrigaitisTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[cfg(feature = "grigaitis_pairwise_booth")]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    super::grigaitis_pairwise_booth::GrigaitisTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[cfg(feature = "grigaitis_pairwise_cpdlh")]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    super::grigaitis_pairwise_cpdlh::GrigaitisTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[cfg(not(any(
    feature = "bgmw",
    feature = "sppark",
    feature = "wbits",
    feature = "grigaitis_pairwise",
    feature = "grigaitis_pairwise_booth",
    feature = "grigaitis_pairwise_cpdlh"
)))]
#[derive(Debug, Clone)]
pub struct EmptyTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    fr_marker: core::marker::PhantomData<TFr>,
    g1_marker: core::marker::PhantomData<TG1>,
    g1_fp_marker: core::marker::PhantomData<TG1Fp>,
    g1_affine_marker: core::marker::PhantomData<TG1Affine>,
    g1_affine_add_marker: core::marker::PhantomData<TG1ProjAddAffine>,
}

#[cfg(not(any(
    feature = "bgmw",
    feature = "sppark",
    feature = "wbits",
    feature = "grigaitis_pairwise",
    feature = "grigaitis_pairwise_booth",
    feature = "grigaitis_pairwise_cpdlh"
)))]
impl<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
    EmptyTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    fn new(_: &[TG1], _: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        Ok(None)
    }

    pub fn multiply_batch(&self, _: &[Vec<TFr>]) -> Vec<TG1> {
        panic!("This function must not be called")
    }

    pub fn multiply_sequential(&self, _: &[TFr]) -> TG1 {
        panic!("This function must not be called")
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, _: &[TFr]) -> TG1 {
        panic!("This function must not be called")
    }
}

#[cfg(not(any(
    feature = "bgmw",
    feature = "sppark",
    feature = "wbits",
    feature = "grigaitis_pairwise",
    feature = "grigaitis_pairwise_booth",
    feature = "grigaitis_pairwise_cpdlh"
)))]
pub type PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> =
    EmptyTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>;

#[allow(clippy::type_complexity)]
pub fn precompute<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>(
    points: &[TG1],
    matrix: &[Vec<TG1>],
) -> Result<Option<PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>>, String>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    PrecomputationTable::<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>::new(points, matrix)
}
