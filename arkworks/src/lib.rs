pub type Pairing = blst::Pairing;
pub type Fp = blst::blst_fp;
pub type Fp12 = blst::blst_fp12;
pub type Fp6 = blst::blst_fp6;
pub type Fr = blst::blst_fr;
pub type P1 = blst::blst_p1;
pub type P1Affine = blst::blst_p1_affine;
pub type P2 = blst::blst_p2;
pub type P2Affine = blst::blst_p2_affine;
pub type Scalar = blst::blst_scalar;
pub type Uniq = blst::blst_uniq;

pub mod das;
pub mod eip_4844;
pub mod fft;
pub mod fft_g1;
pub mod fk20_proofs;
pub mod consts;
pub mod kzg_proofs;
pub mod kzg_types;
pub mod poly;
pub mod recover;
pub mod utils;
pub mod zero_poly;

trait Eq<T> {
    fn equals(&self, other: &T) -> bool;
}

trait Inverse<T> {
    fn inverse(&self) -> T;
}

trait Zero<T> {
    fn is_zero(&self) -> bool;
}

impl Eq<P1> for P1 {
    fn equals(&self, other: &P1) -> bool {
        self.x.l.eq(&other.x.l) && self.y.l.eq(&other.x.l) && self.z.l.eq(&other.x.l)
    }
}

impl Eq<Fr> for Fr {
    fn equals(&self, other: &Fr) -> bool {
        self.l.eq(&other.l)
    }
}

impl Zero<Fr> for Fr {
    fn is_zero(&self) -> bool {
        self.l[0] == 0 && self.l[1] == 0 && self.l[2] == 0 && self.l[3] == 0
    }
}
