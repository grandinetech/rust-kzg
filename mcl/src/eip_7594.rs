extern crate alloc;

use kzg::EcBackend;

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fp::FsFp;
use crate::types::g1::FsG1;
use crate::types::g1::FsG1Affine;
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;
use kzg::c_bindings_eip7594;

use crate::types::fr::FsFr;

pub struct BlstBackend;

impl EcBackend for BlstBackend {
    type Fr = FsFr;
    type G1Fp = FsFp;
    type G1Affine = FsG1Affine;
    type G1 = FsG1;
    type G2 = FsG2;
    type Poly = FsPoly;
    type FFTSettings = FsFFTSettings;
    type KZGSettings = FsKZGSettings;
}

c_bindings_eip7594!(BlstBackend);
