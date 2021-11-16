use zkcrypto::poly::*;
use zkcrypto::fftsettings::*;
use zkcrypto::fftsettings::ZkFFTSettings;
use kzg::{FFTSettingsPoly, Poly};
use zkcrypto::zkfr::{blsScalar, fr_div}; 
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

fn main() {
	// tests to check if some poly functions are working
	let mut rng = StdRng::seed_from_u64(0);
    for _k in 0..10 {
        let multiplicand_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let multiplier_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let out_length: usize = (1 + (rng.next_u64() % 1000)) as usize;

        let mut multiplicand = ZPoly::new(multiplicand_length).unwrap();
        let mut multiplier = ZPoly::new(multiplier_length).unwrap();

        for i in 0..multiplicand_length {
            let coef = blsScalar::rand();
            multiplicand.set_coeff_at(i, &coef);
        }

        for i in 0..multiplier_length {
            let coef = blsScalar::rand();
            multiplier.set_coeff_at(i, &coef);
        }

        //Ensure that the polynomials' orders corresponds to their lengths
        if multiplicand.get_coeff_at(multiplicand.len() - 1).is_zero() {
            let fr_one = blsScalar::one();
            multiplicand.set_coeff_at(multiplicand.len() - 1, &fr_one);
        }

        if multiplier.get_coeff_at(multiplier.len() - 1).is_zero() {
            let fr_one = blsScalar::one();
            multiplier.set_coeff_at(multiplier.len() - 1, &fr_one);
        }
		
		let result2 = ZPoly::mul_direct(&mut multiplicand, &multiplier, out_length);

		let result3 = multiplicand.mul_direct(&multiplier, out_length);
        
		// let result0 = multiplicand.mul_direct(&multiplier, out_length);
		let result0 = ZkFFTSettings::poly_mul_fft(&multiplicand, &multiplier, out_length, None);
        assert!(result0.is_ok());
        let result1 = ZkFFTSettings::poly_mul_fft(&multiplicand, &multiplier, out_length, None);
        assert!(result1.is_ok());
		
		
		
        let actual2 = result2.unwrap();
        let actual3 = result3.unwrap();
	

        let actual0 = result0.unwrap();
        let actual1 = result1.unwrap();

        assert_eq!(actual0.len(), actual1.len());
		
		println!("k =={:?}", _k);
        for i in 0..2 {
            // assert!(actual0.get_coeff_at(i).equals(&actual1.get_coeff_at(i)));
			println!("{:?}  1 == 2", blsScalar::to_u64_arr(&actual0.get_coeff_at(i))); 
			println!("{:?}", blsScalar::to_u64_arr(&actual1.get_coeff_at(i))); 
        	println!(""); 
			 
			println!("{:?}  2 == 3", blsScalar::to_u64_arr(&actual2.get_coeff_at(i))); 
			println!("{:?}", blsScalar::to_u64_arr(&actual3.get_coeff_at(i))); 
			println!("");
			
			 

		}
		
    }
	
}