use kzg::{FFTSettings, FFTSettingsPoly, Fr, Poly};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

pub fn create_poly_of_length_ten<TFr: Fr, TPoly: Poly<TFr>>() {
    let poly = TPoly::new(10);
    assert_eq!(poly.len(), 10);
}

pub fn poly_eval_check<TFr: Fr, TPoly: Poly<TFr>>() {
    let n: usize = 10;
    let mut poly = TPoly::new(n);
    for i in 0..n {
        let fr = TFr::from_u64((i + 1) as u64);
        poly.set_coeff_at(i, &fr);
    }
    let expected = TFr::from_u64((n * (n + 1) / 2) as u64);
    let actual = poly.eval(&TFr::one());
    assert!(expected.equals(&actual));
}

pub fn poly_eval_0_check<TFr: Fr, TPoly: Poly<TFr>>() {
    let n: usize = 7;
    let a: usize = 597;
    let mut poly = TPoly::new(n);
    for i in 0..n {
        let fr = TFr::from_u64((i + a) as u64);
        poly.set_coeff_at(i, &fr);
    }
    let expected = TFr::from_u64(a as u64);
    let actual = poly.eval(&TFr::zero());
    assert!(expected.equals(&actual));
}

pub fn poly_eval_nil_check<TFr: Fr, TPoly: Poly<TFr>>() {
    let n: usize = 0;
    let poly = TPoly::new(n);
    let actual = poly.eval(&TFr::one());
    assert!(actual.equals(&TFr::zero()));
}

pub fn poly_inverse_simple_0<TFr: Fr, TPoly: Poly<TFr>>() {
    // 1 / (1 - x) = 1 + x + x^2 + ...
    let d: usize = 16;
    let mut p = TPoly::new(2);
    p.set_coeff_at(0, &TFr::one());
    p.set_coeff_at(1, &TFr::one());
    p.set_coeff_at(1, &TFr::negate(&p.get_coeff_at(1)));
    let result = p.inverse(d);
    assert!(result.is_ok());
    let q = result.unwrap();
    for i in 0..d {
        assert!(q.get_coeff_at(i).is_one());
    }
}

pub fn poly_inverse_simple_1<TFr: Fr, TPoly: Poly<TFr>>() {
    // 1 / (1 + x) = 1 - x + x^2 - ...
    let d: usize = 16;
    let mut p = TPoly::new(2);
    p.set_coeff_at(0, &TFr::one());
    p.set_coeff_at(1, &TFr::one());
    let result = p.inverse(d);
    assert!(result.is_ok());
    let q = result.unwrap();
    for i in 0..d {
        let mut tmp = q.get_coeff_at(i);
        if i & 1 != 0 {
            tmp = TFr::negate(&tmp);
        }
        assert!(tmp.is_one());
    }
}

fn test_data(a: usize, b: usize) -> Vec<i32> {
    // (x^2 - 1) / (x + 1) = x - 1
    let test_0_0: Vec<i32> = vec![-1, 0, 1];
    let test_0_1: Vec<i32> = vec![1, 1];
    let test_0_2: Vec<i32> = vec![-1, 1];

    // (12x^3 - 11x^2 + 9x + 18) / (4x + 3) = 3x^2 - 5x + 6
    let test_1_0: Vec<i32> = vec![18, 9, -11, 12];
    let test_1_1: Vec<i32> = vec![3, 4];
    let test_1_2: Vec<i32> = vec![6, -5, 3];

    // (x + 1) / (x^2 - 1) = nil
    let test_2_0: Vec<i32> = vec![1, 1];
    let test_2_1: Vec<i32> = vec![-1, 0, 2];
    let test_2_2: Vec<i32> = vec![];

    // (10x^2 + 20x + 30) / 10 = x^2 + 2x + 3
    let test_3_0: Vec<i32> = vec![30, 20, 10];
    let test_3_1: Vec<i32> = vec![10];
    let test_3_2: Vec<i32> = vec![3, 2, 1];

    // (x^2 + x) / (x + 1) = x
    let test_4_0: Vec<i32> = vec![0, 1, 1];
    let test_4_1: Vec<i32> = vec![1, 1];
    let test_4_2: Vec<i32> = vec![0, 1];

    // (x^2 + x + 1) / 1 = x^2 + x + 1
    let test_5_0: Vec<i32> = vec![1, 1, 1];
    let test_5_1: Vec<i32> = vec![1];
    let test_5_2: Vec<i32> = vec![1, 1, 1];

    // (x^2 + x + 1) / (0x + 1) = x^2 + x + 1
    let test_6_0: Vec<i32> = vec![1, 1, 1];
    let test_6_1: Vec<i32> = vec![1, 0]; // The highest coefficient is zero
    let test_6_2: Vec<i32> = vec![1, 1, 1];

    let test_data: [[Vec<i32>; 3]; 7] = [
        [test_0_0, test_0_1, test_0_2],
        [test_1_0, test_1_1, test_1_2],
        [test_2_0, test_2_1, test_2_2],
        [test_3_0, test_3_1, test_3_2],
        [test_4_0, test_4_1, test_4_2],
        [test_5_0, test_5_1, test_5_2],
        [test_6_0, test_6_1, test_6_2],
    ];

    test_data[a][b].clone()
}

fn new_test_poly<TFr: Fr, TPoly: Poly<TFr>>(coeffs: &Vec<i32>) -> TPoly {
    let mut p = TPoly::new(coeffs.len());
    for (i, &coeff) in coeffs.iter().enumerate() {
        if coeff >= 0 {
            let c = TFr::from_u64(coeff as u64);
            p.set_coeff_at(i, &c);
        } else {
            let c = TFr::from_u64((-coeff) as u64);
            let negc = c.negate();
            p.set_coeff_at(i, &negc);
        }
    }
    p
}

pub fn poly_test_div<TFr: Fr, TPoly: Poly<TFr>>() {
    for i in 0..6 {
        let divided_data = test_data(i, 0);
        let divisor_data = test_data(i, 1);
        let expected_data = test_data(i, 2);
        let mut dividend: TPoly = new_test_poly(&divided_data);
        let divisor: TPoly = new_test_poly(&divisor_data);
        let expected: TPoly = new_test_poly(&expected_data);

        let result = dividend.div(&divisor);

        assert!(result.is_ok());
        let actual = result.unwrap();

        assert_eq!(expected.len(), actual.len());
        for i in 0..actual.len() {
            assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
        }
    }
}

pub fn poly_div_by_zero<TFr: Fr, TPoly: Poly<TFr>>() {
    //Arrange
    let coeffs: Vec<i32> = vec![1, 1];
    let mut dividend: TPoly = new_test_poly(&coeffs);
    let divisor = TPoly::new(0);

    //Act
    let result = dividend.div(&divisor);

    //Assert
    assert!(result.is_err());
}

pub fn poly_mul_direct_test<TFr: Fr, TPoly: Poly<TFr>>() {
    let coeffs0: Vec<i32> = vec![3, 4];
    let mut multiplicand: TPoly = new_test_poly(&coeffs0);

    let coeffs1: Vec<i32> = vec![6, -5, 3];
    let mut multiplier: TPoly = new_test_poly(&coeffs1);

    let coeffs2: Vec<i32> = vec![18, 9, -11, 12];
    let expected: TPoly = new_test_poly(&coeffs2);

    let result0 = multiplicand.mul_direct(&multiplier, 4);
    assert!(result0.is_ok());
    let actual0 = result0.unwrap();

    for i in 0..actual0.len() {
        assert!(expected.get_coeff_at(i).equals(&actual0.get_coeff_at(i)))
    }

    //Check commutativity
    let result1 = multiplier.mul_direct(&multiplicand, 4);
    assert!(result1.is_ok());
    let actual1 = result1.unwrap();

    for i in 0..actual1.len() {
        assert!(expected.get_coeff_at(i).equals(&actual1.get_coeff_at(i)))
    }
}

pub fn poly_mul_fft_test<
    TFr: Fr,
    TPoly: Poly<TFr>,
    TFTTSettings: FFTSettings<TFr> + FFTSettingsPoly<TFr, TPoly, TFTTSettings>,
>() {
    let coeffs: Vec<i32> = vec![3, 4];
    let multiplicand: TPoly = new_test_poly(&coeffs);

    let coeffs: Vec<i32> = vec![6, -5, 3];
    let multiplier: TPoly = new_test_poly(&coeffs);

    let coeffs: Vec<i32> = vec![18, 9, -11, 12];
    let expected: TPoly = new_test_poly(&coeffs);

    let result0 = TFTTSettings::poly_mul_fft(&multiplicand, &multiplier, 4, None);
    assert!(result0.is_ok());
    let actual0 = result0.unwrap();

    for i in 0..actual0.len() {
        assert!(expected.get_coeff_at(i).equals(&actual0.get_coeff_at(i)))
    }

    //Check commutativity
    let result1 = TFTTSettings::poly_mul_fft(&multiplier, &multiplicand, 4, None);
    assert!(result1.is_ok());
    let actual1 = result1.unwrap();

    for i in 0..actual1.len() {
        assert!(expected.get_coeff_at(i).equals(&actual1.get_coeff_at(i)))
    }
}

pub fn poly_mul_random<
    TFr: Fr,
    TPoly: Poly<TFr>,
    TFTTSettings: FFTSettings<TFr> + FFTSettingsPoly<TFr, TPoly, TFTTSettings>,
>() {
    let mut rng = StdRng::seed_from_u64(0);
    for _k in 0..256 {
        let multiplicand_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let multiplier_length: usize = (1 + (rng.next_u64() % 1000)) as usize;
        let out_length: usize = (1 + (rng.next_u64() % 1000)) as usize;

        let mut multiplicand = TPoly::new(multiplicand_length);
        let mut multiplier = TPoly::new(multiplier_length);

        for i in 0..multiplicand_length {
            let coef = TFr::rand();
            multiplicand.set_coeff_at(i, &coef);
        }

        for i in 0..multiplier_length {
            let coef = TFr::rand();
            multiplier.set_coeff_at(i, &coef);
        }

        //Ensure that the polynomials' orders corresponds to their lengths
        if multiplicand.get_coeff_at(multiplicand.len() - 1).is_zero() {
            let fr_one = Fr::one();
            multiplicand.set_coeff_at(multiplicand.len() - 1, &fr_one);
        }

        if multiplier.get_coeff_at(multiplier.len() - 1).is_zero() {
            let fr_one = Fr::one();
            multiplier.set_coeff_at(multiplier.len() - 1, &fr_one);
        }

        let result0 = multiplicand.mul_direct(&multiplier, out_length);
        assert!(result0.is_ok());
        let result1 = TFTTSettings::poly_mul_fft(&multiplicand, &multiplier, out_length, None);
        assert!(result1.is_ok());

        let actual0 = result0.unwrap();
        let actual1 = result1.unwrap();

        assert_eq!(actual0.len(), actual1.len());

        for i in 0..actual0.len() {
            assert!(actual0.get_coeff_at(i).equals(&actual1.get_coeff_at(i)));
        }
    }
}

pub fn poly_div_random<TFr: Fr, TPoly: Poly<TFr>>() {
    let mut rng = StdRng::seed_from_u64(0);
    for _k in 0..256 {
        let dividend_length: usize = (2 + (rng.next_u64() % 1000)) as usize;
        let divisor_length: usize = 1 + ((rng.next_u64() as usize) % dividend_length);

        let mut dividend = TPoly::new(dividend_length);
        let mut divisor = TPoly::new(divisor_length);

        for i in 0..dividend_length {
            let coef = TFr::rand();
            dividend.set_coeff_at(i, &coef);
        }

        for i in 0..divisor_length {
            let coef = TFr::rand();
            divisor.set_coeff_at(i, &coef);
        }

        //Ensure that the polynomials' orders corresponds to their lengths
        if dividend.get_coeff_at(dividend.len() - 1).is_zero() {
            let fr_one = Fr::one();
            dividend.set_coeff_at(dividend.len() - 1, &fr_one);
        }

        if divisor.get_coeff_at(divisor.len() - 1).is_zero() {
            let fr_one = Fr::one();
            divisor.set_coeff_at(divisor.len() - 1, &fr_one);
        }

        let result0 = dividend.long_div(&divisor);
        assert!(result0.is_ok());
        let result1 = dividend.fast_div(&divisor);
        assert!(result1.is_ok());

        let actual0 = result0.unwrap();
        let actual1 = result1.unwrap();

        assert_eq!(actual0.len(), actual1.len());

        for i in 0..actual0.len() {
            assert!(actual0.get_coeff_at(i).equals(&actual1.get_coeff_at(i)));
        }
    }
}

pub fn poly_div_long_test<TFr: Fr, TPoly: Poly<TFr>>() {
    for i in 0..7 {
        // Tests are designed to throw an exception when last member is 0
        if i == 6 {
            continue;
        }

        let divided_data = test_data(i, 0);
        let divisor_data = test_data(i, 1);
        let expected_data = test_data(i, 2);
        let mut dividend: TPoly = new_test_poly(&divided_data);
        let divisor: TPoly = new_test_poly(&divisor_data);
        let expected: TPoly = new_test_poly(&expected_data);

        let actual = dividend.long_div(&divisor).unwrap();

        assert_eq!(expected.len(), actual.len());
        for i in 0..actual.len() {
            assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
        }
    }
}

pub fn poly_div_fast_test<TFr: Fr, TPoly: Poly<TFr>>() {
    for i in 0..7 {
        // Tests are designed to throw an exception when last member is 0
        if i == 6 {
            continue;
        }

        let divided_data = test_data(i, 0);
        let divisor_data = test_data(i, 1);
        let expected_data = test_data(i, 2);
        let mut dividend: TPoly = new_test_poly(&divided_data);
        let divisor: TPoly = new_test_poly(&divisor_data);
        let expected: TPoly = new_test_poly(&expected_data);

        let actual = dividend.fast_div(&divisor).unwrap();

        assert_eq!(expected.len(), actual.len());
        for i in 0..actual.len() {
            assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
        }
    }
}
