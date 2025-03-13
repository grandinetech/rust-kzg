use std::ffi::c_void;

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, G1};

#[derive(Debug)]
pub struct SpparkPrecomputation<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    // table holds a reference to value, initialized in C land. It should be never dereferenced
    pub table: *mut c_void,

    fr_marker: core::marker::PhantomData<TFr>,
    g1_marker: core::marker::PhantomData<TG1>,
    g1_fp_marker: core::marker::PhantomData<TG1Fp>,
    g1_affine_marker: core::marker::PhantomData<TG1Affine>,
    g1_proj_add_affine_marker: core::marker::PhantomData<TG1ProjAddAffine>,
}

unsafe impl<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> Sync
    for SpparkPrecomputation<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
}

unsafe impl<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine> Send
    for SpparkPrecomputation<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
}

impl<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
    SpparkPrecomputation<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    pub fn new(_: &[TG1], _: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        // TODO: currently no trait-based implementation for sppark msm, maybe in future
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

    pub fn from_ptr(table: *mut c_void) -> Self {
        Self {
            table,

            fr_marker: core::marker::PhantomData::<TFr>,
            g1_marker: core::marker::PhantomData::<TG1>,
            g1_fp_marker: core::marker::PhantomData::<TG1Fp>,
            g1_affine_marker: core::marker::PhantomData::<TG1Affine>,
            g1_proj_add_affine_marker: core::marker::PhantomData::<TG1ProjAddAffine>,
        }
    }
}
