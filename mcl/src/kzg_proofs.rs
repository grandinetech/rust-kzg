extern crate alloc;

use crate::mcl_methods::mcl_gt;
use crate::types::fp::FsFp;
use crate::types::g1::FsG1;
use crate::types::{fr::FsFr, g1::FsG1Affine};

use crate::types::g1::FsG1ProjAddAffine;

use kzg::msm::{msm_impls::msm, precompute::PrecomputationTable};

use crate::types::g2::FsG2;

use kzg::PairingVerify;

impl PairingVerify<FsG1, FsG2> for FsG1 {
    fn verify(a1: &FsG1, a2: &FsG2, b1: &FsG1, b2: &FsG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

pub fn g1_linear_combination(
    out: &mut FsG1,
    points: &[FsG1],
    scalars: &[FsFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>>,
) {
    *out = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(
        points,
        scalars,
        len,
        precomputation,
    );
}

pub fn pairings_verify(a1: &FsG1, a2: &FsG2, b1: &FsG1, b2: &FsG2) -> bool {
    todo!()
}
