extern crate alloc;

use crate::mcl_methods::{mcl_gt, pairing};
use crate::types::fp::MclFp;
use crate::types::g1::MclG1;
use crate::types::{fr::MclFr, g1::MclG1Affine};

use crate::types::g1::FsG1ProjAddAffine;

use kzg::msm::{msm_impls::msm, precompute::PrecomputationTable};

use crate::types::g2::MclG2;

use kzg::PairingVerify;

impl PairingVerify<MclG1, MclG2> for MclG1 {
    fn verify(a1: &MclG1, a2: &MclG2, b1: &MclG1, b2: &MclG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

pub fn g1_linear_combination(
    out: &mut MclG1,
    points: &[MclG1],
    scalars: &[MclFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<MclFr, MclG1, MclFp, MclG1Affine>>,
) {
    *out = msm::<MclG1, MclFp, MclG1Affine, FsG1ProjAddAffine, MclFr>(
        points,
        scalars,
        len,
        precomputation,
    );
}

pub fn pairings_verify(a1: &MclG1, a2: &MclG2, b1: &MclG1, b2: &MclG2) -> bool {
    // Todo: make optimization
    let mut gt0 = mcl_gt::default();
    pairing(&mut gt0, &a1.0, &a2.0);

    let mut gt1 = mcl_gt::default();
    pairing(&mut gt1, &b1.0, &b2.0);

    gt0 == gt1
}
