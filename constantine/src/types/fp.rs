use constantine_sys as constantine;
use constantine_sys::bls12_381_fp;
use kzg::G1Fp;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct CtFp(pub bls12_381_fp);
impl G1Fp for CtFp {
    const ONE: Self = Self(bls12_381_fp {
        limbs: [
            8505329371266088957,
            17002214543764226050,
            6865905132761471162,
            8632934651105793861,
            6631298214892334189,
            1582556514881692819,
        ],
    });
    const ZERO: Self = Self(bls12_381_fp {
        limbs: [0, 0, 0, 0, 0, 0],
    });
    const BLS12_381_RX_P: Self = Self(bls12_381_fp {
        limbs: [
            8505329371266088957,
            17002214543764226050,
            6865905132761471162,
            8632934651105793861,
            6631298214892334189,
            1582556514881692819,
        ],
    });

    fn inverse(&self) -> Option<Self> {
        let mut out: Self = *self;
        unsafe {
            constantine::ctt_bls12_381_fp_inv(&mut out.0, &self.0);
        }
        Some(out)
    }

    fn square(&self) -> Self {
        let mut out: Self = Default::default();
        unsafe {
            constantine::ctt_bls12_381_fp_square(&mut out.0, &self.0);
        }
        out
    }

    fn double(&self) -> Self {
        let mut out: Self = Default::default();
        unsafe {
            constantine::ctt_bls12_381_fp_double(&mut out.0, &self.0);
        }
        out
    }

    fn from_underlying_arr(arr: &[u64; 6]) -> Self {
        Self(bls12_381_fp { limbs: *arr })
    }

    fn neg_assign(&mut self) {
        unsafe {
            constantine::ctt_bls12_381_fp_neg_in_place(&mut self.0);
        }
    }

    fn mul_assign_fp(&mut self, b: &Self) {
        unsafe {
            constantine::ctt_bls12_381_fp_mul_in_place(&mut self.0, &b.0);
        }
    }

    fn sub_assign_fp(&mut self, b: &Self) {
        unsafe {
            constantine::ctt_bls12_381_fp_sub_in_place(&mut self.0, &b.0);
        }
    }

    fn add_assign_fp(&mut self, b: &Self) {
        unsafe {
            constantine::ctt_bls12_381_fp_add_in_place(&mut self.0, &b.0);
        }
    }
}
