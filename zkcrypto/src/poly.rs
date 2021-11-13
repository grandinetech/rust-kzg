//! This module provides an implementation of polinomials over bls12_381::Scalar
pub use super::{ZPoly, BlsScalar};
pub use kzg::{FFTFr, Poly, Fr, FFTSettings};
use crate::zkfr::{blsScalar, fr_div}; 
//use crate::Fr;
use crate::utils::*;
use crate::fftsettings::{ZkFFTSettings};
use crate::consts::*;
// use crate::fft_fr::*;
use std::convert::TryInto;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgPoly {
    pub coeffs: Vec<blsScalar>
}

impl ZPoly {
	fn new_poly (size: usize) -> Self {
		Self {coeffs: vec![<blsScalar as Fr>::default(); size]}
	}

	/// Checks if the given polynomial is zero.
    	pub(crate) fn is_zero(&self) -> bool {
        	self.coeffs.is_empty()
            		|| self.coeffs.iter().all(|coeff| coeff == &BlsScalar::zero())
    	}

    	/// Constructs a new polynomial from a list of coefficients.
    	///
    	/// # Panics
    	/// When the length of the coeffs is zero.
    	pub(crate) fn from_coefficients_vec(coeffs: Vec<blsScalar>) -> Self {
        	let mut result = Self { coeffs };
        	// While there are zeros at the end of the coefficient vector, pop them
        	// off.
        	result.truncate_leading_zeros();
        	// Check that either the coefficients vec is empty or that the last
        	// coeff is non-zero.
        	assert!(result
            		.coeffs
            		.last()
            		.map_or(true, |coeff| coeff != &blsScalar::zero()));

        	result
    	}

    	/// Returns the degree of the [`Polynomial`].
    	pub(crate) fn degree(&self) -> usize {
        	if self.is_zero() {
            		return 0;
        	}
        	assert!(self
            		.coeffs
            		.last()
            		.map_or(false, |coeff| coeff != &blsScalar::zero()));
        	self.coeffs.len() - 1
    	}

    	fn truncate_leading_zeros(&mut self) {
        	while self
            		.coeffs
            		.last()
            		.map_or(false, |c| c == &blsScalar::zero())
        	{
            		self.coeffs.pop();
        	}
    	}

    	/// Divides a [`Polynomial`] by x-z using Ruffinis method.
    	pub fn ruffini(&self, z: blsScalar) -> KzgPoly {
        	let mut quotient: Vec<blsScalar> = Vec::with_capacity(self.degree());
        	let mut k = blsScalar::zero();

        	// Reverse the results and use Ruffini's method to compute the quotient
        	// The coefficients must be reversed as Ruffini's method
        	// starts with the leading coefficient, while Polynomials
        	// are stored in increasing order i.e. the leading coefficient is the
        	// last element
        	for coeff in self.coeffs.iter().rev() {
            		let t = coeff + k;
            		quotient.push(t);
            		k = z * t;
        	}

        	// Pop off the last element, it is the remainder term
        	// For PLONK, we only care about perfect factors
        	quotient.pop();

        	// Reverse the results for storage in the Polynomial struct
        	quotient.reverse();
        	KzgPoly::from_coefficients_vec(quotient)
    	}
}

impl Poly<blsScalar> for ZPoly {
    fn default() -> Self {
        // Self {
            // coeffs: vec![<blsScalar as Fr>::default(); 4] // blsScalar::default()
			
		// }
		Self::new(1).unwrap()
		
    }
	fn new(size: usize) -> Result<Self, String> {
        Ok(Self{coeffs: vec![<blsScalar as Fr>::default(); size]}) // blsScalar::default()
    }
	
	fn get_coeff_at(&self, i: usize) -> blsScalar {
		self.coeffs[i]
	}

    fn set_coeff_at(&mut self, i: usize, x: &blsScalar) {
		self.coeffs[i] = x.clone()
	}

    fn get_coeffs(&self) -> &[blsScalar] {
		&self.coeffs
	}

    fn len(&self) -> usize {
		self.coeffs.len()
	}

    fn eval(&self, x: &blsScalar) -> blsScalar {
		if self.coeffs.len() == 0 {
            return blsScalar::zero();
        } else if x.is_zero() {
            return self.coeffs[0].clone();
        }

        let mut ret = self.coeffs[self.coeffs.len() - 1].clone();
        let mut i = self.coeffs.len() - 2;
        loop {
            let temp = ret.mul(&x);
            ret = temp.add(&self.coeffs[i]);

            if i == 0 {
                break;
            }
            i -= 1;
        }

        return ret; 
	
	}

    fn scale(&mut self) {
        let scale_factor = blsScalar::from_u64(SCALE_FACTOR);
        let inv_factor = scale_factor.inverse();

        let mut factor_power = blsScalar::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&inv_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn unscale(&mut self) {
        let scale_factor = blsScalar::from_u64(SCALE_FACTOR);

        let mut factor_power = blsScalar::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&scale_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

	fn inverse(&mut self, new_len: usize) -> Result<Self, String> { // +
		// let mut poly = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); new_len] };  //::new(new_len).unwrap();
		
		// poly_inverse(self, &mut poly) // galbut reikia pirma poly siust??
		if self.coeffs.len() == 0 {
            return Err(String::from("Can't inverse a zero-length poly"));
        } else if self.coeffs[0].is_zero() {
            return Err(String::from("First coefficient of polynomial mustn't be zero"));
        }

        let mut ret = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); new_len] };
        // If the input polynomial is constant, the remainder of the series is zero
        if self.coeffs.len() == 1 {
            ret.coeffs[0] = self.coeffs[0].eucl_inverse();

            // for i in 1..new_len {
                // ret.coeffs[i] = <blsScalar as Fr>::zero();
            // }

            return Ok(ret);
        }

        let maxd = new_len - 1;


        // Max space for multiplications is (2 * length - 1)
        // let scale: usize = log2_pow2(next_power_of_two(2 * new_len - 1));
        //let scale: usize = log2_pow2(next_power_of_two(2 * new_len - 1));
        // let fs = ZkFFTSettings::new(scale).unwrap();

        // To store intermediate results
        // let mut tmp0 = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); new_len] };
        // let mut tmp1 = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); new_len] };

        // Base case for d == 0
        ret.coeffs[0] = self.coeffs[0].eucl_inverse();
        let mut d: usize = 0;
        let mut mask: usize = 1 << log2_u64(maxd);
        while mask != 0 {
            d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
            mask >>= 1; // mask = mask >> 1

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = min_u64(d + 1, self.coeffs.len() + new_len - 1).unwrap();
            let mut tmp0 = poly_mul(self, &ret, len_temp).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..tmp0.len() {
				// let tempFr = tmp0.coeffs[i].clone();
                // tmp0.coeffs[i] = tempFr.negate();
				tmp0.coeffs[i] = tmp0.coeffs[i].negate();
            }
            let fr_two = <blsScalar as Fr>::from_u64(2);
            tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

            // c.(2 - b.c) -> tmp1;
            let tmp1 = poly_mul(&ret, &tmp0, d + 1).unwrap();

			//out_length = tmp1.len();
            for i in 0..tmp1.coeffs.len() {
				ret.coeffs[i] = tmp1.coeffs[i];
                // ret.coeffs.push(tmp1.coeffs[i]);
            }
        }

        if d + 1 != new_len {
            return Err(String::from("d+1 is bad"));
        }

        Ok(ret)
		
		
		
	}

    fn div(&mut self, x: &Self) -> Result<Self, String> {
		
		let ret = new_poly_div(&self, &x);
		//let res = new_poly_div(&self, &x).unwrap();
		

		ret
	}
	
	fn long_div(&mut self, x: &Self) -> Result<Self, String> {
		
		for i in 0..5 {
			println!("self = {:?}", self.get_coeff_at(i));
		}
		
		for i in 0..5 {
			println!("x = {:?}", x.get_coeff_at(i));
		}
		
		let ret = poly_long_div(&self, &x);
		let res = poly_long_div(&self, &x).unwrap();
		for i in 0..5 {
			println!("res = {:?}", res.get_coeff_at(i));
		}
		
		ret
	}

    fn fast_div(&mut self, x: &Self) -> Result<Self, String>{
	
		for i in 0..5 {
			println!("self = {:?}", self.get_coeff_at(i));
		}
		
		for i in 0..5 {
			println!("x = {:?}", x.get_coeff_at(i));
		}
		
		let res = poly_fast_div(&self, &x).unwrap();
		for i in 0..5 {
			println!("res = {:?}", res.get_coeff_at(i));
		}
		let ret = poly_fast_div(&self, &x);
		ret
	}

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String>{
	
		// for i in 0..3 {	
			// println!("SELF = {:?}", self.get_coeff_at(i));
		// }
		// for i in 0..3 {	
			// println!("X = {:?}", x.get_coeff_at(i));
		// }
		// let ret = poly_mul_direct(&self, &x, len);
		// return ret;
		poly_mul_direct(self, x, len)
	}
}

pub fn poly_long_div(dividend: &ZPoly, divisor: &ZPoly) -> Result<ZPoly, String> { // +
    
    // assert!(divisor.coeffs.len() > 0);
    // assert!(!divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

	if divisor.coeffs.len() == 0 {
        return Err(String::from("Can't divide by zero"));
    } 
	// else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
        // return Err(String::from("Highest coefficient must be non-zero"));
    // }
	// println!("dividend is {:?}", dividend.coeffs.len());
	// println!("divisor is {:?}", divisor.coeffs.len());
	
    let mut a_pos = dividend.coeffs.len() - 1;
    let b_pos = divisor.coeffs.len() - 1;
    let mut diff: isize = (a_pos as isize) - (b_pos as isize);

    let result = poly_quotient_length(&dividend, &divisor); // ar gera f-ija
    assert!(result.is_ok());
    let out_length = result.unwrap();
	let mut out: ZPoly = ZPoly {coeffs: vec![<blsScalar as Default>::default(); out_length]}; // { coeffs: Vec::default() };

    // assert!(out.coeffs.len() >= out_length);

    if out_length == 0 {
        return Ok(out);
    }

    
    let mut a = vec![<blsScalar as Default>::default(); dividend.coeffs.len()];// blsScalar::default()
    for i in 0..a.len() { // dividend.coeffs.len()
		a[i] = dividend.coeffs[i];
        // a.push(dividend.coeffs[i]);
    }

    while diff > 0 {
        let result = fr_div(&a[a_pos], &divisor.coeffs[b_pos]);
        // assert!(result.is_ok());
        out.coeffs[diff as usize] = result.unwrap();

        
            for i in 0..(b_pos + 1) {
                let tmp = out.coeffs[diff as usize].mul(&divisor.coeffs[i]);
				let tmp = a[(diff as usize) + i].sub(&tmp);
                a[(diff as usize) + i] = tmp;
				// a[diff + i] = a[diff + i].sub(&tmp);
				
            }
        diff -= 1;
        a_pos -= 1;
    }
    let result = fr_div(&a[a_pos], &divisor.coeffs[b_pos]);
    // assert!(result.is_ok());
    out.coeffs[0] = result.unwrap();

    Ok(out)
}

// pub fn poly_flip(input: &ZPoly) -> Result<ZPoly, String> {
    // let mut output = ZPoly { coeffs: Vec::default() };
    // for i in 0..input.coeffs.len() {
        // output.coeffs.push(input.coeffs[input.coeffs.len() - i - 1]);
    // }
    // Ok(output)
// }

pub fn poly_inverse(b: &ZPoly, out: &mut ZPoly) -> Result<ZPoly, String> { // +
    // assert!(b.coeffs.len() > 0);
    // assert!(!b.coeffs[0].is_zero());
	// assert!(out.coeffs.len() > 0);

    let mut val2 = ZPoly {coeffs: vec![blsScalar::zero(); out.coeffs.len()] }; // { coeffs: Vec::default() };
    // If the input polynomial is constant, the remainder of the series is zero
    if b.coeffs.len() == 1 {
        // is this right?
        val2.coeffs[0] = b.coeffs[0].inverse(); // eucl_inverse?
        //out.coeffs[0] = b.coeffs[0].inverse();
		// for i in 1..val2.coeffs.len() {
            // out.coeffs[i] = blsScalar::zero(); // not sure if this is right
			// val2.coeffs[i] = blsScalar::zero();
		// }
        return Ok(val2);
    }

    let maxd = out.coeffs.len() - 1;
    let mut d: usize = 0;

    // let scale: usize = log2_pow2(next_power_of_two(2 * out.coeffs.len() - 1));

    // let fs: ZkFFTSettings = ZkFFTSettings::new(scale).unwrap();
    // let fs: ZkFFTSettings = ZkFFTSettings::from_scale(scale).unwrap();


    // let mut tmp0 = ZPoly::new_poly(out.coeffs.len()); //{ coeffs: Vec::default() };
    // let mut tmp1 = ZPoly::new_poly(out.coeffs.len()); //{ coeffs: Vec::default() }; 
	
	let mut val1 = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); out.coeffs.len()] };

    // out.coeffs[0] = b.coeffs[0].inverse(); // eucl_inverse? is this good?
	val1.coeffs[0] = b.coeffs[0].inverse();
	
	val2.coeffs[0] = b.coeffs[0].eucl_inverse();

    let mut mask: usize = 1 << log2_u64(maxd);
    while mask != 0 {
        d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
        mask = mask >> 1;

        let len_temp = min_u64(d + 1, b.coeffs.len() + out.coeffs.len() - 1).unwrap();
        // if d + 1 < b.coeffs.len() + out.coeffs.len() - 1 {
            // len_temp = d + 1;
        // } else {
            // len_temp = b.coeffs.len() + out.coeffs.len() - 1
        // }

        let mut tmp0 = poly_mul(&b, &val2, len_temp).unwrap();

        for i in 0..tmp0.coeffs.len() {
           // let cloned_fr = tmp0.coeffs[i].clone();
            tmp0.coeffs[i] = tmp0.coeffs[i].negate(); //cloned_fr.negate();
        }
        let fr_two = blsScalar::from_u64(2);
		
		// is this good?
        tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

        let tmp1 = poly_mul(&val2, &tmp0, d + 1).unwrap();
		
		// let out_length = tmp1.len();
        for i in 0..tmp1.coeffs.len() {
        //    out.coeffs.push(tmp1.coeffs[i]);
			// val2.coeffs.push(tmp1.coeffs[i]);
			val2.coeffs[i] = tmp1.coeffs[i];
		}
    }
    // assert!(d + 1 == val.coeffs.len());
	
	
    Ok(val2)
}


pub fn poly_fast_div(dividend: &ZPoly, divisor: &ZPoly) -> Result<ZPoly, String> { // +

    // Dividing by zero is undefined
    // assert!(divisor.coeffs.len() > 0);

    // assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

	if divisor.coeffs.len() == 0 {
        return Err(String::from("Cant divide by zero"));
    } 
	// else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
        // return Err(String::from("Highest coefficient must be non-zero"));
    // }


    let m: usize = dividend.coeffs.len() - 1;
    let n: usize = divisor.coeffs.len() - 1;

    if n > m {
        return Ok(ZPoly {coeffs: Vec::new() });//ZPoly::default()  );  //{ coeffs: Vec::default() }
    }

    // assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    // let mut out = ZPoly::default(); // { coeffs: Vec::default() };
    if divisor.coeffs.len() == 1 {
		let mut out: ZPoly = ZPoly { coeffs: vec![blsScalar::zero(); dividend.len()] };
        for i in 0..out.len() { // dividend.coeffs.len()
            // out.coeffs.push(fr_div(&dividend.coeffs[i], &divisor.coeffs[0]).unwrap());
			out.coeffs[i] = fr_div(&dividend.coeffs[i], &divisor.coeffs[0]).unwrap();
		}
        return Ok(out);
    }

    let a_flip = poly_flip(&dividend).unwrap(); //{ coeffs: Vec::default() };
    let mut b_flip = poly_flip(&divisor).unwrap(); //{ coeffs: Vec::default() };

    // a_flip = poly_flip(&dividend).unwrap();
    // b_flip = poly_flip(&divisor).unwrap();

    // let mut inv_b_flip = ZPoly::new_poly(m - n + 1); // { coeffs: Vec::default() };
    // inv_b_flip = poly_inverse(&b_flip, &mut inv_b_flip).unwrap();

	let inv_b_flip = b_flip.inverse(m - n + 1).unwrap(); // could be bad


    // let mut q_flip = ZPoly::default(); // { coeffs: Vec::default() };

    let q_flip = poly_mul(&a_flip, &inv_b_flip, m - n + 1).unwrap();
	
    let out = poly_flip(&q_flip).unwrap();

    Ok(out)
}

pub fn poly_flip(out: &ZPoly) -> Result<ZPoly, String> { // +
    let mut ret = ZPoly { coeffs: vec![<blsScalar as Default>::default(); out.len()] };
    for i in 0..out.len() {
        // ret.coeffs[i] = out.coeffs[out.coeffs.len() - i - 1]
		ret.coeffs[out.len() - i - 1] = out.coeffs[i]
    }

    Ok(ret)
}


pub fn new_poly_div(dividend_: &ZPoly, divisor_: &ZPoly) -> Result<ZPoly, String> { // +

    let result = poly_norm(dividend_);
    assert!(result.is_ok());
    let dividend: ZPoly = result.unwrap();

    let result = poly_norm(divisor_);
    assert!(result.is_ok());
    let divisor: ZPoly = result.unwrap();

    if divisor.coeffs.len() >= dividend.coeffs.len() || divisor.coeffs.len() < 128 { // Tunable paramter
        let result = poly_long_div(&dividend, &divisor);
        // assert!(result.is_ok());
        result
    } else {
        let result = poly_fast_div(&dividend, &divisor);
        // assert!(result.is_ok());
        result
    }
}

pub fn poly_norm(p: &ZPoly) -> Result<ZPoly, String> { // +
    let mut ret = p.clone();
    let mut temp_len: usize = ret.coeffs.len();
    while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
        temp_len -= 1;
    }
    if temp_len == 0 {
        ret.coeffs = Vec::new();
    }
	else {
		ret.coeffs = ret.coeffs[0..temp_len].to_vec();
    }
    Ok(ret)
}

pub fn poly_quotient_length(dividend: &ZPoly, divisor: &ZPoly) -> Result<usize, String> { // +
    if dividend.coeffs.len() >= divisor.coeffs.len() {
        return Ok(dividend.coeffs.len() - divisor.coeffs.len() + 1);
    }
	else{
		Ok(0)
	}
}

pub fn pad(input: &ZPoly, length: usize) -> Vec<blsScalar> { // +
    // let num: usize = min_u64(n_in, n_out).unwrap();
    //let mut output: Vec<blsScalar> = Vec::default();
	// let mut output = input.coeffs.to_vec();
	let mut out = vec![blsScalar::zero(); length]; 
    // for i in 0..num {
        // output[i] = input[i].clone();
    // }
    // for i in num..n_out {
        // output[i] = blsScalar::zero();
    // }
	for i in 0..min_u64(input.len(), length).unwrap() {
		out[i] = input.coeffs[i];
	}
    out
}
// fs_ Option<&ZkFFTSettings> ar reikia & ar ne?
pub fn poly_mul_fft(out: usize, a: &ZPoly, b: &ZPoly ) -> Result<ZPoly, String> { // +

    let a_len = min_u64(a.coeffs.len(), out).unwrap();
    let b_len = min_u64(b.coeffs.len(), out).unwrap();
    let length = next_power_of_two(a_len + b_len - 1);
	// pasitikrinti log2_pow2_pow2, next_power_of_two, ... funkcijas
	let ft_size: usize = log2_pow2(length);
	
    let fs = ZkFFTSettings::new(ft_size).unwrap(); //ZkFFTSettings::new(0).unwrap();
    
	assert!(length <= fs.max_width);
	
	
	
	let a_pad = pad(&a, length); // gal check pad
    let b_pad = pad(&b, length);
	
	
	
	
	
	let a_fft = fs.fft_fr(&a_pad, false).unwrap(); // gal perzvelk pagal c-kzg fft_fr
	let b_fft = fs.fft_fr(&b_pad, false).unwrap();
	
	
	// let mut ab_fft = a_pad;
	let mut ab_fft = vec![<blsScalar as Default>::default(); length];
	
    for i in 0..length {
		ab_fft[i] = a_fft[i].mul(&b_fft[i]);
    }
	
    let ab = fs.fft_fr(&ab_fft, true).unwrap();

    let mut output = ZPoly {coeffs: vec![blsScalar::zero(); out] };
	let data_len = min_u64(out, length).unwrap(); // ar tikrai geri kintamieji?
	
	// for i in 0..3 {	
		// println!("a(1,2,3) inside mulfft = {:?}", a.get_coeff_at(i));
		// println!("b(1,2,3) inside mulfft = {:?}", b.get_coeff_at(i));
	// }
	// for i in 0..3 {	
		// println!("a_pad = {:?}", a_pad[i]);
		// }
	// for i in 0..3 {	
		// println!("b_pad = {:?}", b_pad[i]);
	// }
	// for i in 0..3 {	
		// println!("a_fft = {:?}", a_fft[i]);
		// }
	// for i in 0..3 {	
		// println!("b_fft = {:?}", b_fft[i]);
	// }
	// for i in 0..3 {	
			// println!("ab_fft = {:?}", ab_fft[i]);
		// }
	// for i in 0..3 {	
			// println!("ab = {:?}", ab[i]);
		// }
	
	for i in 0..data_len {
        output.coeffs[i] = ab[i];//push(ab[i]);
    }
	
		// for i in 0..output.len() {	
			// println!("output = {:?}", output.get_coeff_at(i));
		// }


    return Ok(output);
}

pub fn poly_mul_direct(a: &ZPoly, b: &ZPoly, output_len: usize) -> Result<ZPoly, String> { // +

    let a_degree: usize = a.coeffs.len() - 1;
    let b_degree: usize = b.coeffs.len() - 1;
    let mut output = ZPoly { coeffs: vec![blsScalar::zero(); output_len] };
	
	
	

    for i in 0..(a_degree + 1) {
        let mut j: usize = 0;
        while (j <= b_degree) && ((i + j) < output_len) {
			let tmp = a.coeffs[i].mul(&b.coeffs[j]);//get_coeff_at(i).mul(&b.get_coeff_at(j));
			let tmp = output.coeffs[i + j].add(&tmp);
			output.coeffs[i + j] = tmp;
			// output.coeffs[i + j] = output.coeffs[i + j].add(&tmp);
			j += 1;
        }
    }
	// for i in 0..3 {	
			// println!("a(mul_direct = {:?}", a.get_coeff_at(i));
			// println!("b(mul_direct) = {:?}", b.get_coeff_at(i));
		// }
	// for i in 0..3 {
		// println!("output(mul_direct) = {:?}", output.get_coeff_at(i));
		// }

    Ok(output)
}

// pub fn poly_mul_(a: &ZPoly, b: &ZPoly, _fs: &ZkFFTSettings, output_len: usize) -> Result<ZPoly, String> {
    // if a.coeffs.len() < 64 || b.coeffs.len() < 64 || output_len < 128 { // Tunable parameter
        // return poly_mul_direct(&a, &b, output_len);
    // } else {
        // return poly_mul_fft(output_len, a, b);
    // }
// }

pub fn poly_mul(a: &ZPoly, b: &ZPoly, output_len: usize) -> Result<ZPoly, String> { // +
	if (a.len() < 64) || (b.len() < 64) || (output_len < 128) {
		return poly_mul_direct(&a, &b, output_len); 
	}
	else {
		return poly_mul_fft(output_len, &a, &b);
	}
    // let fft_settings = ZkFFTSettings::new(0).unwrap();
    // return poly_mul_(&a, &b, &fft_settings, output_len);
}

