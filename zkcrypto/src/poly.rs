//! This module provides an implementation of polinomials over bls12_381::Scalar
pub use super::{ZPoly, BlsScalar};
pub use kzg::{FFTFr, Poly, Fr, FFTSettings};
use crate::zkfr::{blsScalar, fr_div}; 
//use crate::Fr;
use crate::utils::*;
use crate::fftsettings::{ZkFFTSettings};
use crate::consts::*;
// use crate::fft_fr::*;


#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgPoly {
    pub coeffs: Vec<blsScalar>
}

impl ZPoly {
	fn new_poly (size: usize) -> Self {
		Self {coeffs: vec![<blsScalar as Fr>::default(); size]}
	}

}

impl Poly<blsScalar> for ZPoly {
    fn default() -> Self {
        Self {
            coeffs: vec![<blsScalar as Fr>::default(); 4] // blsScalar::default()
        }
		
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

	fn inverse(&mut self, new_len: usize) -> Result<Self, String> {
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

            for i in 1..new_len {
                ret.coeffs[i] = <blsScalar as Fr>::zero();
            }

            return Ok(ret);
        }

        let maxd = new_len - 1;

        // Max space for multiplications is (2 * length - 1)
        let scale: usize = log2_pow2(next_power_of_two(2 * new_len - 1));
        let scale: usize = log2_pow2(next_power_of_two(2 * new_len - 1));
        let fs = ZkFFTSettings::new(scale).unwrap();

        // To store intermediate results
        let mut tmp0 = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); new_len] };
        let mut tmp1 = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); new_len] };

        // Base case for d == 0
        ret.coeffs[0] = self.coeffs[0].eucl_inverse();
        let mut d: usize = 0;
        let mut mask: usize = 1 << log2_u64(maxd);
        while mask != 0 {
            d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
            mask = mask >> 1;

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = min_u64(d + 1, self.coeffs.len() + new_len - 1).unwrap();
            tmp0 = poly_mul(self, &ret, len_temp).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..tmp0.coeffs.len() {
                tmp0.coeffs[i] = tmp0.coeffs[i].negate();
            }
            let fr_two = <blsScalar as Fr>::from_u64(2);
            tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

            // c.(2 - b.c) -> tmp1;
            tmp1 = poly_mul(&ret, &tmp0, d + 1).unwrap();

            // output->length = tmp1.length;
            for i in 0..tmp1.coeffs.len() {
                ret.coeffs.push(tmp1.coeffs[i]);
            }
        }

        if d + 1 != new_len {
            return Err(String::from(""));
        }

        Ok(ret)
		
		
		
	}

    fn div(&mut self, x: &Self) -> Result<Self, String> {
		todo!()
	}
	
	fn long_div(&mut self, x: &Self) -> Result<Self, String> {
		todo!()
	}

    fn fast_div(&mut self, x: &Self) -> Result<Self, String>{
		todo!()
	}

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String>{
		todo!()
	}
    
	
    fn destroy(&mut self) {}
}

pub fn poly_long_div(dividend: &ZPoly, divisor: &ZPoly) -> Result<ZPoly, String> {
    
    assert!(divisor.coeffs.len() > 0);
    assert!(!divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    let mut out: ZPoly = ZPoly::default(); // { coeffs: Vec::default() };
    let mut a_pos = dividend.coeffs.len();
    let b_pos = divisor.coeffs.len();
    let mut diff = a_pos - b_pos;

    let result = poly_quotient_length(&dividend, &divisor);
    assert!(result.is_ok());
    let out_length = result.unwrap();

    assert!(out.coeffs.len() >= out_length);

    if out_length == 0 {
        return Ok(out);
    }

    
    let mut a = vec![Default::default(); dividend.coeffs.len()];// blsScalar::default()
    for i in 0..dividend.coeffs.len() {
    
        a.push(dividend.coeffs[i]);
    }

    while diff > 0 {
        let result = fr_div(&a[a_pos], &divisor.coeffs[b_pos]);
        assert!(result.is_ok());
        out.coeffs[diff] = result.unwrap();

        
            for i in 0..(b_pos + 1) {
                let tmp = out.coeffs[diff].mul(&divisor.coeffs[i]);
                a[diff + i] = a[diff + i].sub(&tmp);
            }
        diff -= 1;
        a_pos -= 1;
    }
    let result = fr_div(&a[a_pos], &divisor.coeffs[b_pos]);
    assert!(result.is_ok());
    out.coeffs[0] = result.unwrap();

    Ok(out)
}

pub fn poly_flip(input: &ZPoly) -> Result<ZPoly, String> {
    let mut output = ZPoly { coeffs: Vec::default() };
    for i in 0..input.coeffs.len() {
        output.coeffs.push(input.coeffs[input.coeffs.len() - i - 1]);
    }
    Ok(output)
}

pub fn poly_inverse(b: &ZPoly, out: &mut ZPoly) -> Result<ZPoly, String> {
    assert!(b.coeffs.len() > 0);
    assert!(!b.coeffs[0].is_zero());
	assert!(out.coeffs.len() > 0);

    let mut val2 = ZPoly {coeffs: vec![<blsScalar as Fr>::default(); out.coeffs.len()] }; // { coeffs: Vec::default() };
    // If the input polynomial is constant, the remainder of the series is zero
    if b.coeffs.len() == 1 {
        // is this right?
        val2.coeffs[0] = b.coeffs[0].inverse(); // eucl_inverse?
        out.coeffs[0] = b.coeffs[0].inverse();
		for i in 1..val2.coeffs.len() {
            out.coeffs[i] = blsScalar::zero(); // not sure if this is right
			val2.coeffs[i] = blsScalar::zero();
		}
        return Ok(val2);
    }

    let maxd = out.coeffs.len() - 1;
    let mut d: usize = 0;

    let scale: usize = log2_pow2(next_power_of_two(2 * out.coeffs.len() - 1));

    let fs: ZkFFTSettings = ZkFFTSettings::new(scale).unwrap();
    // let fs: ZkFFTSettings = ZkFFTSettings::from_scale(scale).unwrap();


    let mut tmp0 = ZPoly::new_poly(out.coeffs.len()); //{ coeffs: Vec::default() };
    let mut tmp1 = ZPoly::new_poly(out.coeffs.len()); //{ coeffs: Vec::default() }; 
	
	let mut val1 = ZPoly { coeffs: vec![<blsScalar as Fr>::default(); out.coeffs.len()] };

    out.coeffs[0] = b.coeffs[0].inverse(); // eucl_inverse? is this good?
	val1.coeffs[0] = b.coeffs[0].inverse();

    let mut mask: usize = 1 << log2_u64(maxd);
    while mask != 0 {
        d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
        mask = mask >> 1;

        let len_temp = min_u64(d + 1, b.coeffs.len() + out.coeffs.len()).unwrap();
        // if d + 1 < b.coeffs.len() + out.coeffs.len() - 1 {
            // len_temp = d + 1;
        // } else {
            // len_temp = b.coeffs.len() + out.coeffs.len() - 1
        // }

        tmp0 = poly_mul_(&b, &out, &fs, len_temp).unwrap();

        for i in 0..tmp0.coeffs.len() {
           // let cloned_fr = tmp0.coeffs[i].clone();
            tmp0.coeffs[i] = tmp0.coeffs[i].negate(); //cloned_fr.negate();
        }
        let fr_two = blsScalar::from_u64(2);
		
		// is this good?
        tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

        tmp1 = poly_mul_(&out, &tmp0, &fs, d + 1).unwrap();
        for i in 0..tmp1.coeffs.len() {
        //    out.coeffs.push(tmp1.coeffs[i]);
			val1.coeffs.push(tmp1.coeffs[i]);
		}
    }
    // assert!(d + 1 == val.coeffs.len());
	
	
    Ok(val1)
}


pub fn poly_fast_div(dividend: &ZPoly, divisor: &ZPoly) -> Result<ZPoly, String> {

    // Dividing by zero is undefined
    assert!(divisor.coeffs.len() > 0);

    assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    let m: usize = dividend.coeffs.len() - 1;
    let n: usize = divisor.coeffs.len() - 1;

    if n > m {
        return Ok(ZPoly::default()  );  //{ coeffs: Vec::default() }
    }

    assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    let mut out = ZPoly::default(); // { coeffs: Vec::default() };
    if divisor.coeffs.len() == 1 {
        for i in 0..dividend.coeffs.len() {
            out.coeffs.push(fr_div(&dividend.coeffs[i], &divisor.coeffs[0]).unwrap());
        }
        return Ok(out);
    }

    let mut a_flip = ZPoly::default(); //{ coeffs: Vec::default() };
    let mut b_flip = ZPoly::default(); //{ coeffs: Vec::default() };

    a_flip = poly_flip(&dividend).unwrap();
    b_flip = poly_flip(&divisor).unwrap();

    let mut inv_b_flip = ZPoly::new_poly(m - n + 1); // { coeffs: Vec::default() };
    inv_b_flip = poly_inverse(&b_flip, &mut inv_b_flip).unwrap();

    let mut q_flip = ZPoly::default(); // { coeffs: Vec::default() };

    q_flip = poly_mul(&a_flip, &inv_b_flip, m - n + 1).unwrap();

    out = poly_flip(&q_flip).unwrap();

    Ok(out)
}

pub fn new_poly_div(dividend_: &ZPoly, divisor_: &ZPoly) -> Result<ZPoly, String> {

    let result = poly_norm(dividend_);
    assert!(result.is_ok());
    let dividend: ZPoly = result.unwrap();

    let result = poly_norm(divisor_);
    assert!(result.is_ok());
    let divisor: ZPoly = result.unwrap();

    if divisor.coeffs.len() >= dividend.coeffs.len() || divisor.coeffs.len() < 128 { // Tunable paramter
        let result = poly_long_div(&dividend, &divisor);
        assert!(result.is_ok());
        result
    } else {
        let result = poly_fast_div(&dividend, &divisor);
        assert!(result.is_ok());
        result
    }
}

pub fn poly_norm(p: &ZPoly) -> Result<ZPoly, String> {
    let mut ret = p.clone();
    let mut temp_len: usize = ret.coeffs.len();
    while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
        temp_len -= 1;
    }
    if temp_len == 0 {
        ret.coeffs = Vec::default();
    }

    Ok(ret)
}

pub fn poly_quotient_length(dividend: &ZPoly, divisor: &ZPoly) -> Result<usize, String> {
    if dividend.coeffs.len() >= divisor.coeffs.len() {
        return Ok(dividend.coeffs.len() - divisor.coeffs.len() + 1);
    }
	else{
		Ok(0)
	}
}

pub fn pad(input: &ZPoly, n_in: usize, n_out: usize) -> Result<Vec<blsScalar>, String> {
    // let num: usize = min_u64(n_in, n_out).unwrap();
    //let mut output: Vec<blsScalar> = Vec::default();
	let mut output = input.coeffs.to_vec();
    // for i in 0..num {
        // output[i] = input[i].clone();
    // }
    // for i in num..n_out {
        // output[i] = blsScalar::zero();
    // }
	for _i in input.coeffs.len()..n_out {
		output.push(blsScalar::zero())
	}
    Ok(output)
}
// fs_ Option<&ZkFFTSettings> ar reikia & ar ne?
pub fn poly_mul_fft(out: usize, a: &ZPoly, b: &ZPoly) -> Result<ZPoly, String> {

    let a_len = min_u64(a.coeffs.len(), out).unwrap();
    let b_len = min_u64(b.coeffs.len(), out).unwrap();
    let length = next_power_of_two(a_len + b_len - 1);

	let ft_size: usize = log2_pow2(length);
	
    let mut fs = ZkFFTSettings::new(ft_size).unwrap(); //ZkFFTSettings::new(0).unwrap();
    // match fs_ {
		// Some(x) => fs = x.clone(),
		// None => {
			// let scale: usize = log2_pow2(length);
			// fs = new_fft_settings(scale);
	
        
		// } 
    // }
	assert!(length <= fs.max_width);

	// let mut a_pad: Vec<blsScalar> = Vec::default();
	// let mut b_pad: Vec<blsScalar> = Vec::default(); 
	// let mut a_fft: Vec<blsScalar> = Vec::default(); 
	// let mut b_fft: Vec<blsScalar> = Vec::default();
	
	let a_pad = pad(&a, a_len, length).unwrap();
    let b_pad = pad(&b, b_len, length).unwrap();
	
	// patikrinti fft_fr
    // a_fft = fft_fr(&a_pad, false, &fs).unwrap();
    // b_fft = fft_fr(&b_pad, false, &fs).unwrap();
	
	let a_fft = fs.fft_fr(&a_pad, false).unwrap();
	let b_fft = fs.fft_fr(&b_pad, false).unwrap();
	
	
	let mut ab_fft = a_pad;
	
    for i in 0..length {
		ab_fft[i] = a_fft[i].mul(&b_fft[i]);
    }
	// pratesti
    let ab = fs.fft_fr(&ab_fft, true).unwrap();

    let mut output = ZPoly {coeffs: Vec::default() };
	let data_len = min_u64(out, length).unwrap(); // ar tikrai geri kintamieji?
	
	for i in 0..data_len {
        output.coeffs.push(ab[i]);
    }
	
	for _ in data_len..out {
		output.coeffs.push(blsScalar::zero());
	}

    return Ok(output);
}

pub fn poly_mul_direct(a: &ZPoly, b: &ZPoly, output_len: usize) -> Result<ZPoly, String> {

    let a_degree: usize = a.coeffs.len() - 1;
    let b_degree: usize = b.coeffs.len() - 1;
    let mut output = ZPoly { coeffs: Vec::default() };

    for _ in 0..output_len {
        output.coeffs.push(blsScalar::zero());
		// output.set_coeff_at(k, &blsScalar::zero());
    }

    for i in 0..(a_degree + 1) {
        let mut j: usize = 0;
        while j <= b_degree && (i + j) < output.coeffs.len() {
			let tmp = a.coeffs[i].mul(&b.coeffs[j]);//get_coeff_at(i).mul(&b.get_coeff_at(j));
			output.coeffs[i + j] = output.coeffs[i + j].add(&tmp);
			j += 1;
        }
    }

    Ok(output)
}

pub fn poly_mul_(a: &ZPoly, b: &ZPoly, fs: &ZkFFTSettings, output_len: usize) -> Result<ZPoly, String> {
    if a.coeffs.len() < 64 || b.coeffs.len() < 64 || output_len < 128 { // Tunable parameter
        return poly_mul_direct(&a, &b, output_len);
    } else {
        return poly_mul_fft(output_len, a, b);
    }
}

pub fn poly_mul(a: &ZPoly, b: &ZPoly, output_len: usize) -> Result<ZPoly, String> {
    let fft_settings = ZkFFTSettings::new(0).unwrap();
    return poly_mul_(&a, &b, &fft_settings, output_len);
}

