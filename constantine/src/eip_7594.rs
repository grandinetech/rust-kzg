extern crate alloc;

use kzg::EcBackend;

use crate::types::fft_settings::CtFFTSettings;
use crate::types::fp::CtFp;
use crate::types::fr::CtFr;
use crate::types::g1::CtG1;
use crate::types::g1::CtG1Affine;
use crate::types::g1::CtG1ProjAddAffine;
use crate::types::g2::CtG2;
use crate::types::kzg_settings::CtKZGSettings;
use crate::types::poly::CtPoly;

pub struct CtBackend;

impl EcBackend for CtBackend {
    type Fr = CtFr;
    type G1Fp = CtFp;
    type G1Affine = CtG1Affine;
    type G1ProjAddAffine = CtG1ProjAddAffine;
    type G1 = CtG1;
    type G2 = CtG2;
    type Poly = CtPoly;
    type FFTSettings = CtFFTSettings;
    type KZGSettings = CtKZGSettings;
}

#[cfg(feature = "c_bindings")]
kzg::c_bindings_eip7594!(CtBackend);
