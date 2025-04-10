extern crate alloc;

use kzg::EcBackend;

use crate::types::fft_settings::MclFFTSettings;
use crate::types::fp::MclFp;
use crate::types::g1::MclG1;
use crate::types::g1::MclG1Affine;
use crate::types::g1::MclG1ProjAddAffine;
use crate::types::g2::MclG2;
use crate::types::kzg_settings::MclKZGSettings;
use crate::types::poly::MclPoly;

use crate::types::fr::MclFr;

pub struct MclBackend;

impl EcBackend for MclBackend {
    type Fr = MclFr;
    type G1Fp = MclFp;
    type G1Affine = MclG1Affine;
    type G1ProjAddAffine = MclG1ProjAddAffine;
    type G1 = MclG1;
    type G2 = MclG2;
    type Poly = MclPoly;
    type FFTSettings = MclFFTSettings;
    type KZGSettings = MclKZGSettings;
}

kzg::c_bindings_eip7594!(MclBackend);
