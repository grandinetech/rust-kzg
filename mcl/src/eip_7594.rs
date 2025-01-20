extern crate alloc;

use kzg::EcBackend;

use crate::types::fft_settings::MclFFTSettings;
use crate::types::fp::MclFp;
use crate::types::g1::MclG1;
use crate::types::g1::FsG1Affine;
use crate::types::g2::MclG2;
use crate::types::kzg_settings::MclKZGSettings;
use crate::types::poly::MclPoly;
use kzg::c_bindings_eip7594;

use crate::types::fr::MclFr;

pub struct MclBackend;

impl EcBackend for MclBackend {
    type Fr = MclFr;
    type G1Fp = MclFp;
    type G1Affine = FsG1Affine;
    type G1 = MclG1;
    type G2 = MclG2;
    type Poly = MclPoly;
    type FFTSettings = MclFFTSettings;
    type KZGSettings = MclKZGSettings;
}

c_bindings_eip7594!(MclBackend);
