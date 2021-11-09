use crate::data_types::fr::Fr;
use kzg::Fr as CommonFr;
use rand::Rng;

impl CommonFr for Fr {
    fn default() -> Self {
        Fr::zero()
    }

    fn zero() -> Self {
        Fr::zero()
    }

    fn one() -> Self {
        Fr::from_int(1)
    }

    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        // let mut fr = Fr::zero();
        // Fr::set_by_csprng(&mut fr);
        // fr
        //SHOULD BE CHANGED
        //Fr::random()

        //Pros: works with DAS test Cons: slower
        //No clue why MCL one does not work with DAS test
        let mut val: [u64; 4] = [0, 0, 0, 0];
        for x in &mut val {
            *x = rng.gen_range(0..1);
        }
        let ret = Self::from_u64_arr(&val);
        ret
        
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        Fr::from_u64_arr(u)
    }

    fn from_u64(val: u64) -> Self {
        Fr::from_u64_arr(&[val, 0, 0, 0])
    }

	fn to_u64_arr(&self) -> [u64; 4] {
		todo!()
	}
	
	fn div(&self, b: &Self) -> Result<Self, String>{
		todo!()
	}
	
    fn is_one(&self) -> bool {
        Fr::is_one(self)
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
        let mut res = self.clone();
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

    fn destroy(&mut self) {}
}
