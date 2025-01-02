extern crate alloc;

use kzg::EcBackend;

use kzg::c_bindings_eip7594;

use crate::kzg_proofs::FFTSettings;
use crate::kzg_proofs::KZGSettings;
use crate::kzg_types::ArkFp;
use crate::kzg_types::ArkFr;
use crate::kzg_types::ArkG1;
use crate::kzg_types::ArkG1Affine;
use crate::kzg_types::ArkG2;
use crate::utils::PolyData;

pub struct ArkBackend;

impl EcBackend for ArkBackend {
    type Fr = ArkFr;
    type G1Fp = ArkFp;
    type G1Affine = ArkG1Affine;
    type G1 = ArkG1;
    type G2 = ArkG2;
    type Poly = PolyData;
    type FFTSettings = FFTSettings;
    type KZGSettings = KZGSettings;
}

c_bindings_eip7594!(ArkBackend);
