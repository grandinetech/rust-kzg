use crate::data_types::fr::Fr;
use kzg::Fr as CommonFr;

impl CommonFr for Fr {
    fn default() -> Self {
        Fr::zero()
    }

    fn null() -> Self {
        Fr::from_u64_arr(&[u64::MAX, u64::MAX, u64::MAX, u64::MAX / 3])
    }

    fn zero() -> Self {
        Fr::zero()
    }

    fn one() -> Self {
        Fr::from_int(1)
    }

    fn rand() -> Self {
        Fr::random()
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        Fr::from_u64_arr(u)
    }

    fn from_u64(val: u64) -> Self {
        Fr::from_u64_arr(&[val, 0, 0, 0])
    }

	fn to_u64_arr(&self) -> [u64; 4] {
        Fr::to_u64_arr(self)
	}
	
	fn div(&self, b: &Self) -> Result<Self, String>{
        let mut res = Fr::zero();
        Fr::div(&mut res, self, b);
        Ok(res)
	}
	
    fn is_one(&self) -> bool {
        Fr::is_one(self)
    }

    fn is_null(&self) -> bool {
        let temp = Fr::null();
        self.equals(&temp)
    }

    fn is_zero(&self) -> bool {
        Fr::is_zero(self)
    }

    fn sqr(&self) -> Self {
        let mut res = Fr::zero();
        Fr::sqr(&mut res, self);
        res
    }

    fn pow(&self, n: usize) -> Self {
        //No idea if this works
        let mut res = *self;
        for _ in 1 .. n {
            res = res * *self;
        }
        res
    }

    fn mul(&self, b: &Self) -> Self {
        let mut res = Fr::zero();
        Fr::mul(&mut res, self, b);
        res
    }

    fn add(&self, b: &Self) -> Self {
        let mut res = Fr::zero();
        Fr::add(&mut res, self, b);
        res
    }

    fn sub(&self, b: &Self) -> Self {
        let mut res = Fr::zero();
        Fr::sub(&mut res, self, b);
        res
    }

    fn eucl_inverse(&self) -> Self {
        todo!()
    }

    fn negate(&self) -> Self {
        self.get_neg()
    }

    fn inverse(&self) -> Self {
        self.inverse()
    }

    fn equals(&self, b: &Self) -> bool {
        Fr::eq(self, b)
    }
}
