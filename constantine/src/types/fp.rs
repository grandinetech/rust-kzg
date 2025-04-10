use constantine_sys as constantine;
use constantine_sys::bls12_381_fp;
use core::fmt::{Debug, Formatter};
use kzg::G1Fp;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct CtFp(pub bls12_381_fp);

impl PartialEq for CtFp {
    fn eq(&self, other: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_fp_is_eq(&self.0, &other.0) != 0 }
    }
}

impl Debug for CtFp {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "CtFp({:?})", self.0.limbs)
    }
}

impl G1Fp for CtFp {
    fn one() -> Self {
        Self(bls12_381_fp {
            limbs: [
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
        Self(bls12_381_fp {
            limbs: [0, 0, 0, 0, 0, 0],
        })
    }

    fn bls12_381_rx_p() -> Self {
        Self(bls12_381_fp {
            limbs: [
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
        unsafe {
            Self(bls12_381_fp {
                limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(*arr),
            })
        }
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

    fn mul3(&self) -> Self {
        const THREE: CtFp = CtFp(bls12_381_fp {
            limbs: [
                17157870155352091297,
                9692872460839157767,
                5726366251156250088,
                11420128032487956561,
                9069687087735597977,
                1000072309349998725,
            ],
        });

        self.mul_fp(&THREE)
    }
}
