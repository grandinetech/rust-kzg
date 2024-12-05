use blst::blst_fp;
use kzg::G1Fp;

use crate::mcl_methods::{mclBnFp_add, mclBnFp_inv, mclBnFp_mul, mclBnFp_neg, mclBnFp_sqr, mclBnFp_sub, try_init_mcl, Fp};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsFp(pub Fp);
impl G1Fp for FsFp {
    fn one() -> Self {
        try_init_mcl();
            
        Self(Fp {
            d: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        })
    }

    fn zero() -> Self {
        try_init_mcl();

        Self(Fp {
            d: [0, 0, 0, 0, 0, 0],
        })
    }

    fn bls12_381_rx_p() -> Self {
        try_init_mcl();

        Self(Fp {
            d: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        })
    }

    fn inverse(&self) -> Option<Self> {
        try_init_mcl();

        let mut out: Self = *self;
        Fp::inv(&mut out.0, &self.0);

        Some(out)
    }

    fn square(&self) -> Self {
        try_init_mcl();

        let mut out: Self = *self;
        Fp::sqr(&mut out.0, &self.0);

        out
    }

    fn double(&self) -> Self {
        try_init_mcl();
        
        let mut out: Self = Default::default();

        unsafe {
            mclBnFp_add(&mut out.0, &self.0, &self.0);
        }

        out
    }

    fn from_underlying_arr(arr: &[u64; 6]) -> Self {
        try_init_mcl();

        Self(Fp { d: *arr })
    }

    fn neg_assign(&mut self) {
        try_init_mcl();
        
        unsafe {
            mclBnFp_neg(&mut self.0, &self.0);
        }
    }

    fn mul_assign_fp(&mut self, b: &Self) {
        try_init_mcl();
        self.0 *= &b.0;
    }

    fn sub_assign_fp(&mut self, b: &Self) {
        try_init_mcl();
        self.0 -= &b.0;
    }

    fn add_assign_fp(&mut self, b: &Self) {
        try_init_mcl();
        self.0 += &b.0;
    }
}
