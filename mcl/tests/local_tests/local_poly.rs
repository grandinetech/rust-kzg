use kzg::{Fr, Poly};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use rust_kzg_blst::types::fr::FsFr;
use rust_kzg_blst::types::poly::FsPoly;

pub fn create_poly_of_length_ten() {
    let poly = FsPoly::new(10);
    assert_eq!(poly.len(), 10);
}

pub fn poly_pad_works_rand() {
    let mut rng = StdRng::seed_from_u64(0);

    for _k in 0..256 {
        let poly_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let mut poly = FsPoly::new(poly_length);
        for i in 0..poly.len() {
            poly.set_coeff_at(i, &FsFr::rand());
        }

        let padded_poly = poly.pad(1000);
        for i in 0..poly_length {
            assert!(padded_poly.get_coeff_at(i).equals(&poly.get_coeff_at(i)));
        }
        for i in poly_length..1000 {
            assert!(padded_poly.get_coeff_at(i).equals(&Fr::zero()));
        }
    }
}

pub fn poly_eval_check() {
    let n: usize = 10;
    let mut poly = FsPoly::new(n);
    for i in 0..n {
        let fr = FsFr::from_u64((i + 1) as u64);
        poly.set_coeff_at(i, &fr);
    }
    let expected = FsFr::from_u64((n * (n + 1) / 2) as u64);
    let actual = poly.eval(&FsFr::one());
    assert!(expected.equals(&actual));
}

pub fn poly_eval_0_check() {
    let n: usize = 7;
    let a: usize = 597;
    let mut poly = FsPoly::new(n);
    for i in 0..n {
        let fr = FsFr::from_u64((i + a) as u64);
        poly.set_coeff_at(i, &fr);
    }
    let expected = FsFr::from_u64(a as u64);
    let actual = poly.eval(&FsFr::zero());
    assert!(expected.equals(&actual));
}

pub fn poly_eval_nil_check() {
    let n: usize = 0;
    let poly = FsPoly::new(n);
    let actual = poly.eval(&FsFr::one());
    assert!(actual.equals(&FsFr::zero()));
}

pub fn poly_inverse_simple_0() {
    // 1 / (1 - x) = 1 + x + x^2 + ...
    let d: usize = 16;
    let mut p = FsPoly::new(2);
    p.set_coeff_at(0, &FsFr::one());
    p.set_coeff_at(1, &FsFr::one());
    p.set_coeff_at(1, &FsFr::negate(&p.get_coeff_at(1)));
    let result = p.inverse(d);
    assert!(result.is_ok());
    let q = result.unwrap();
    for i in 0..d {
        assert!(q.get_coeff_at(i).is_one());
    }
}

pub fn poly_inverse_simple_1() {
    // 1 / (1 + x) = 1 - x + x^2 - ...
    let d: usize = 16;
    let mut p = FsPoly::new(2);
    p.set_coeff_at(0, &FsFr::one());
    p.set_coeff_at(1, &FsFr::one());
    let result = p.inverse(d);
    assert!(result.is_ok());
    let q = result.unwrap();
    for i in 0..d {
        let mut tmp = q.get_coeff_at(i);
        if i & 1 != 0 {
            tmp = FsFr::negate(&tmp);
        }
        assert!(tmp.is_one());
    }
}

pub const NUM_TESTS: u64 = 10;

fn test_data(a: usize, b: usize) -> Vec<i64> {
    // (x^2 - 1) / (x + 1) = x - 1
    let test_0_0 = vec![-1, 0, 1];
    let test_0_1 = vec![1, 1];
    let test_0_2 = vec![-1, 1];

    // (12x^3 - 11x^2 + 9x + 18) / (4x + 3) = 3x^2 - 5x + 6
    let test_1_0 = vec![18, 9, -11, 12];
    let test_1_1 = vec![3, 4];
    let test_1_2 = vec![6, -5, 3];

    // (x + 1) / (x^2 - 1) = nil
    let test_2_0 = vec![1, 1];
    let test_2_1 = vec![-1, 0, 2];
    let test_2_2 = vec![];

    // (10x^2 + 20x + 30) / 10 = x^2 + 2x + 3
    let test_3_0 = vec![30, 20, 10];
    let test_3_1 = vec![10];
    let test_3_2 = vec![3, 2, 1];

    // (x^2 + x) / (x + 1) = x
    let test_4_0 = vec![0, 1, 1];
    let test_4_1 = vec![1, 1];
    let test_4_2 = vec![0, 1];

    // (x^2 + x + 1) / 1 = x^2 + x + 1
    let test_5_0 = vec![1, 1, 1];
    let test_5_1 = vec![1];
    let test_5_2 = vec![1, 1, 1];

    // (x^2 + x + 1) / (0x + 1) = x^2 + x + 1
    let test_6_0 = vec![1, 1, 1];
    let test_6_1 = vec![1, 0]; // The highest coefficient is zero
    let test_6_2 = vec![1, 1, 1];

    // (x^3) / (x) = (x^2)
    let test_7_0 = vec![0, 0, 0, 1];
    let test_7_1 = vec![0, 1];
    let test_7_2 = vec![0, 0, 1];

    //
    let test_8_0 = vec![
        236,
        945,
        -297698,
        2489425,
        -18556462,
        -301325440,
        2473062655,
        -20699887353,
    ];
    let test_8_1 = vec![4, 11, -5000, 45541, -454533];
    let test_8_2 = vec![59, 74, -878, 45541];

    // (x^4 + 2x^3 + 3x^2 + 2x + 1) / (-x^2 -x -1) = (-x^2 -x -1)
    let test_9_0 = vec![1, 2, 3, 2, 1];
    let test_9_1 = vec![-1, -1, -1];
    let test_9_2 = vec![-1, -1, -1];

    let test_data = [
        [test_0_0, test_0_1, test_0_2],
        [test_1_0, test_1_1, test_1_2],
        [test_2_0, test_2_1, test_2_2],
        [test_3_0, test_3_1, test_3_2],
        [test_4_0, test_4_1, test_4_2],
        [test_5_0, test_5_1, test_5_2],
        [test_6_0, test_6_1, test_6_2],
        [test_7_0, test_7_1, test_7_2],
        [test_8_0, test_8_1, test_8_2],
        [test_9_0, test_9_1, test_9_2],
    ];

    test_data[a][b].clone()
}

fn new_test_poly(coeffs: &[i64]) -> FsPoly {
    let mut p = FsPoly::new(0);

    for &coeff in coeffs.iter() {
        if coeff >= 0 {
            let c = FsFr::from_u64(coeff as u64);
            p.coeffs.push(c);
        } else {
            let c = FsFr::from_u64((-coeff) as u64);
            let negc = c.negate();
            p.coeffs.push(negc);
        }
    }

    p
}

pub fn poly_div_long_test() {
    for i in 0..9 {
        // Tests are designed to throw an exception when last member is 0
        if i == 6 {
            continue;
        }

        let divided_data = test_data(i, 0);
        let divisor_data = test_data(i, 1);
        let expected_data = test_data(i, 2);
        let mut dividend: FsPoly = new_test_poly(&divided_data);
        let divisor: FsPoly = new_test_poly(&divisor_data);
        let expected: FsPoly = new_test_poly(&expected_data);

        let actual = dividend.long_div(&divisor).unwrap();

        assert_eq!(expected.len(), actual.len());
        for i in 0..actual.len() {
            assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
        }
    }
}

pub fn poly_div_fast_test() {
    for i in 0..9 {
        // Tests are designed to throw an exception when last member is 0
        if i == 6 {
            continue;
        }

        let divided_data = test_data(i, 0);
        let divisor_data = test_data(i, 1);
        let expected_data = test_data(i, 2);
        let mut dividend: FsPoly = new_test_poly(&divided_data);
        let divisor: FsPoly = new_test_poly(&divisor_data);
        let expected: FsPoly = new_test_poly(&expected_data);

        let actual = dividend.fast_div(&divisor).unwrap();

        assert_eq!(expected.len(), actual.len());
        for i in 0..actual.len() {
            assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
        }
    }
}

pub fn test_poly_div_by_zero() {
    let mut dividend = FsPoly::new(2);

    dividend.set_coeff_at(0, &FsFr::from_u64(1));
    dividend.set_coeff_at(1, &FsFr::from_u64(1));

    let divisor = FsPoly::new(0);

    let dummy = dividend.div(&divisor);
    assert!(dummy.is_err());
}

pub fn poly_mul_direct_test() {
    for i in 0..9 {
        let coeffs1 = test_data(i, 2);
        let coeffs2 = test_data(i, 1);
        let coeffs3 = test_data(i, 0);

        let mut multiplicand: FsPoly = new_test_poly(&coeffs1);
        let mut multiplier: FsPoly = new_test_poly(&coeffs2);
        let expected: FsPoly = new_test_poly(&coeffs3);

        let result0 = multiplicand.mul_direct(&multiplier, coeffs3.len()).unwrap();
        for j in 0..result0.len() {
            assert!(expected.get_coeff_at(j).equals(&result0.get_coeff_at(j)))
        }

        // Check commutativity
        let result1 = multiplier.mul_direct(&multiplicand, coeffs3.len()).unwrap();
        for j in 0..result1.len() {
            assert!(expected.get_coeff_at(j).equals(&result1.get_coeff_at(j)))
        }
    }
}

pub fn poly_mul_fft_test() {
    for i in 0..9 {
        // Ignore 0 multiplication case because its incorrect when multiplied backwards
        if i == 2 {
            continue;
        }

        let coeffs1 = test_data(i, 2);
        let coeffs2 = test_data(i, 1);
        let coeffs3 = test_data(i, 0);

        let multiplicand: FsPoly = new_test_poly(&coeffs1);
        let multiplier: FsPoly = new_test_poly(&coeffs2);
        let expected: FsPoly = new_test_poly(&coeffs3);

        let result0 = multiplicand.mul_fft(&multiplier, coeffs3.len()).unwrap();
        for j in 0..result0.len() {
            assert!(expected.get_coeff_at(j).equals(&result0.get_coeff_at(j)))
        }

        // Check commutativity
        let result1 = multiplier.mul_fft(&multiplicand, coeffs3.len()).unwrap();
        for j in 0..result1.len() {
            assert!(expected.get_coeff_at(j).equals(&result1.get_coeff_at(j)))
        }
    }
}

pub fn poly_mul_random() {
    let mut rng = StdRng::seed_from_u64(0);

    for _k in 0..256 {
        let multiplicand_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let mut multiplicand = FsPoly::new(multiplicand_length);
        for i in 0..multiplicand.len() {
            multiplicand.set_coeff_at(i, &FsFr::rand());
        }

        let multiplier_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let mut multiplier = FsPoly::new(multiplier_length);
        for i in 0..multiplier.len() {
            multiplier.set_coeff_at(i, &FsFr::rand());
        }

        if multiplicand.get_coeff_at(multiplicand.len() - 1).is_zero() {
            multiplicand.set_coeff_at(multiplicand.len() - 1, &Fr::one());
        }

        if multiplier.get_coeff_at(multiplier.len() - 1).is_zero() {
            multiplier.set_coeff_at(multiplier.len() - 1, &Fr::one());
        }

        let out_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let q0 = multiplicand.mul_direct(&multiplier, out_length).unwrap();
        let q1 = multiplicand.mul_fft(&multiplier, out_length).unwrap();

        assert_eq!(q0.len(), q1.len());
        for i in 0..q0.len() {
            assert!(q0.get_coeff_at(i).equals(&q1.get_coeff_at(i)));
        }
    }
}

pub fn poly_div_random() {
    let mut rng = StdRng::seed_from_u64(0);
    for _k in 0..256 {
        let dividend_length: usize = (2 + (rng.next_u64() % 1000)) as usize;
        let divisor_length: usize = 1 + ((rng.next_u64() as usize) % dividend_length);

        let mut dividend = FsPoly::new(dividend_length);
        let mut divisor = FsPoly::new(divisor_length);

        for i in 0..dividend_length {
            dividend.set_coeff_at(i, &FsFr::rand());
        }

        for i in 0..divisor_length {
            divisor.set_coeff_at(i, &FsFr::rand());
        }

        //Ensure that the polynomials' orders corresponds to their lengths
        if dividend.get_coeff_at(dividend.len() - 1).is_zero() {
            dividend.set_coeff_at(dividend.len() - 1, &Fr::one());
        }

        if divisor.get_coeff_at(divisor.len() - 1).is_zero() {
            divisor.set_coeff_at(divisor.len() - 1, &Fr::one());
        }

        let result0 = dividend.long_div(&divisor).unwrap();
        let result1 = dividend.fast_div(&divisor).unwrap();

        assert_eq!(result0.len(), result1.len());
        for i in 0..result0.len() {
            assert!(result0.get_coeff_at(i).equals(&result1.get_coeff_at(i)));
        }
    }
}
