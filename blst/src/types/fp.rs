use blst::blst_fp;
use kzg::G1Fp;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsFp(pub blst_fp);
impl G1Fp for FsFp {
    const ONE: Self = Self(blst_fp {
        l: [
            8505329371266088957,
            17002214543764226050,
            6865905132761471162,
            8632934651105793861,
            6631298214892334189,
            1582556514881692819,
        ],
    });
    const ZERO: Self = Self(blst_fp {
        l: [0, 0, 0, 0, 0, 0],
    });
    const BLS12_381_RX_P: Self = Self(blst_fp {
        l: [
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
            blst::blst_fp_inverse(&mut out.0, &self.0);
        }
        Some(out)
    }

    fn square(&self) -> Self {
        let mut out: Self = Default::default();
        unsafe {
            blst::blst_fp_sqr(&mut out.0, &self.0);
        }
        out
    }

    fn double(&self) -> Self {
        let mut out: Self = Default::default();
        unsafe {
            blst::blst_fp_add(&mut out.0, &self.0, &self.0);
        }
        out
    }

    fn from_underlying_arr(arr: &[u64; 6]) -> Self {
        Self(blst_fp { l: *arr })
    }

    fn neg_assign(&mut self) {
        unsafe {
            blst::blst_fp_cneg(&mut self.0, &self.0, true);
        }
    }

    fn mul_assign_fp(&mut self, b: &Self) {
        unsafe {
            blst::blst_fp_mul(&mut self.0, &self.0, &b.0);
        }
    }

    fn sub_assign_fp(&mut self, b: &Self) {
        unsafe {
            blst::blst_fp_sub(&mut self.0, &self.0, &b.0);
        }
    }

    fn add_assign_fp(&mut self, b: &Self) {
        unsafe {
            blst::blst_fp_add(&mut self.0, &self.0, &b.0);
        }
    }
}
