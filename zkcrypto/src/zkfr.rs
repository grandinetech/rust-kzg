// Adds implementation for blsScalar (Fr)

pub use super::{ZPoly, BlsScalar};
use kzg::Fr;
use crate::utils::*;
pub use crate::curve::scalar::Scalar as blsScalar; 

impl Fr for blsScalar {

	fn default() -> Self {
		<blsScalar as Default>::default()
	}
	
	fn zero() -> Self {
		blsScalar::zero()
	}
	
	fn one() -> Self {
		blsScalar::one()
	}

    fn rand() -> Self {
		let val: [u64; 4] = rand::random();
		blsScalar::from_raw(val)
        // let mut ret = Self::default();
        // unsafe {
            // blsScalar::from_raw(val);
			// //blst_fr_from_uint64(&mut ret.0, val.as_ptr());
        // }  
	}
	
	fn from_u64_arr(u: &[u64; 4]) -> Self {
		blsScalar::from_raw(*u)
		
	}
	
	fn from_u64(val: u64) -> Self {
		blsScalar::from(val)
	}
	
	fn is_one(&self) -> bool {
		// assert!(self.eq(&blsScalar::one()));
		self.eq(&blsScalar::one())
		// self == blsScalar::one()
		// let mut val: [u64; 4] = [0; 4];		
		// blsScalar::from_raw(val);
		// return val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0;
	}

    fn is_zero(&self) -> bool{
		self.eq(&blsScalar::zero())
		// self == blsScalar::zero()
        // let mut val: [u64; 4] = [0; 4];	
		// <blsScalar as From<u64>>::from(val.as_mut_ptr());
        // return val[0] == 0 && val[1] == 0 && val[2] == 0 && val[3] == 0;
    }
	
	fn sqr(&self) -> Self {
		blsScalar::square(&self)
	}
	
    fn mul(&self, b: &Self) -> Self {
		let mut ret = <blsScalar as Fr>::default(); // Self::default() is this needed?
			blsScalar::mul(&self, &b) // &b.0 or &ret.0?
	}
	
	fn add(&self, b: &Self) -> Self {
		let mut ret = <blsScalar as Fr>::default(); // Self::default() is this needed?
			blsScalar::add(&self, &b)
	}
	fn sub(&self, b: &Self) -> Self {
		let mut ret = <blsScalar as Fr>::default(); // Self::default() is this needed?
		blsScalar::sub(&self, &b) // for this
	}
	
	fn eucl_inverse(&self) -> Self { 
		//let mut ret = Default::default(); //Self::default()
		// blsScalar::invert(&self).unwrap()
		// self.invert().unwrap()

		let mut ret = blst::blst_fr::default();
		let ToBlst = zk_fr_into_blst_fr(self);
		unsafe {
			blst::blst_fr_eucl_inverse(&mut ret, &ToBlst);
		}
		let output = blst_fr_into_zk_fr(&ret);
		output
	}
	
	fn pow(&self, n: usize) -> Self {
	// unfinished. bls12_381 scalar has pow method. 
	// also for i in 1..n out.sqr();
    let mut tmp = self.clone();
    let mut out = Self::one();
    let mut n2 = n;
    
        loop {
            if n2 & 1 == 1 {
                out.mul(&tmp);
            }
            n2 = n2 >> 1;
            if n2 == 0 {
                break;
            }
            out.sqr();
        }
    
    out
}

	fn negate(&self) -> Self {
		//blsScalar::neg(&self)
		self.neg()
	}
	
	fn inverse(&self) -> Self {
		//let mut ret = <blsScalar as Fr>::default(); // Self::default()
		// Self::invert(&self).unwrap()
		
		// self.invert().unwrap()
		
		let mut ret = blst::blst_fr::default();
		let ToBlst = zk_fr_into_blst_fr(self);
		unsafe {
			blst::blst_fr_inverse(&mut ret, &ToBlst);
		}
		let output = blst_fr_into_zk_fr(&ret);
		output
	}
	
	fn div(&self, b: &Self) -> Result<Self, String> {
		let tmp = b.eucl_inverse();
		let out = self.mul(&tmp);
		Ok(out)
	}
	
	fn equals(&self, other: &Self) -> bool {
		self.eq(other)
	}
	
	fn destroy(&mut self) {}
}

pub fn fr_div(a: &blsScalar, b: &blsScalar) -> Result<blsScalar, String> {

    let tmp = b.inverse();
    let out = a.mul(&tmp);
    Ok(out)
}
