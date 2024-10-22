extern crate alloc;

use kzg::EcBackend;

use kzg::c_bindings_eip7594;

use crate::kzg_proofs::FFTSettings;
use crate::kzg_proofs::KZGSettings;
use crate::kzg_types::ZFp;
use crate::kzg_types::ZFr;
use crate::kzg_types::ZG1Affine;
use crate::kzg_types::ZG1;
use crate::kzg_types::ZG2;
use crate::poly::PolyData;

pub struct ZBackend;

impl EcBackend for ZBackend {
    type Fr = ZFr;
    type G1Fp = ZFp;
    type G1Affine = ZG1Affine;
    type G1 = ZG1;
    type G2 = ZG2;
    type Poly = PolyData;
    type FFTSettings = FFTSettings;
    type KZGSettings = KZGSettings;
}

c_bindings_eip7594!(ZBackend);
