extern crate alloc;

use kzg::EcBackend;

use crate::kzg_proofs::FFTSettings;
use crate::kzg_proofs::KZGSettings;
use crate::kzg_types::ZFp;
use crate::kzg_types::ZFr;
use crate::kzg_types::ZG1Affine;
use crate::kzg_types::ZG1ProjAddAffine;
use crate::kzg_types::ZG1;
use crate::kzg_types::ZG2;
use crate::poly::PolyData;

pub struct ZBackend;

impl EcBackend for ZBackend {
    type Fr = ZFr;
    type G1Fp = ZFp;
    type G1Affine = ZG1Affine;
    type G1ProjAddAffine = ZG1ProjAddAffine;
    type G1 = ZG1;
    type G2 = ZG2;
    type Poly = PolyData;
    type FFTSettings = FFTSettings;
    type KZGSettings = KZGSettings;
}

#[cfg(feature = "c_bindings")]
kzg::c_bindings_eip7594!(ZBackend);
