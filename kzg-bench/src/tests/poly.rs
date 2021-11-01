use kzg::{Fr, Poly, FFTSettings};
// use rand::rngs::StdRng;
// use rand::seq::SliceRandom;

pub fn create_poly_of_length_ten<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    let mut poly = TPoly::new(10).unwrap();
    assert_eq!(poly.len(), 10);
    poly.destroy();
}

pub fn poly_eval_check<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    let n: usize = 10;
    let mut poly = TPoly::new(n).unwrap();
    for i in 0..n {
        let fr = TFr::from_u64((i + 1) as u64);
        poly.set_coeff_at(i, &fr);
    }
    let expected = TFr::from_u64((n * (n + 1) / 2) as u64);
    let actual = poly.eval(&TFr::one());
    assert!(expected.equals(&actual));
    poly.destroy();
}

pub fn poly_eval_0_check<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    let n: usize = 7;
    let a: usize = 597;
    let mut poly = TPoly::new(n).unwrap();
    for i in 0..n {
        let fr = TFr::from_u64((i + a) as u64);
        poly.set_coeff_at(i, &fr);
    }
    let expected = TFr::from_u64(a as u64);
    let actual = poly.eval(&TFr::zero());
    assert!(expected.equals(&actual));
    poly.destroy();
}

pub fn poly_eval_nil_check<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    let n: usize = 0;
    let mut poly = TPoly::new(n).unwrap();
    let actual = poly.eval(&TFr::one());
    assert!(actual.equals(&TFr::zero()));
    poly.destroy();
}

pub fn poly_inverse_simple_0<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    // 1 / (1 - x) = 1 + x + x^2 + ...
    let d: usize = 16;
    let mut p = TPoly::new(2).unwrap();
    p.set_coeff_at(0, &TFr::one());
    p.set_coeff_at(1, &TFr::one());
    p.set_coeff_at(1, &TFr::negate(&p.get_coeff_at(1)));
    let result = p.inverse(d);
    assert!(result.is_ok());
    let mut q = result.unwrap();
    for i in 0..d {
        assert!(q.get_coeff_at(i).is_one());
    }
    q.destroy();
}

pub fn poly_inverse_simple_1<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    // 1 / (1 + x) = 1 - x + x^2 - ...
    let d: usize = 16;
    let mut p = TPoly::new(2).unwrap();
    p.set_coeff_at(0, &TFr::one());
    p.set_coeff_at(1, &TFr::one());
    let result = p.inverse(d);
    assert!(result.is_ok());
    let mut q = result.unwrap();
    for i in 0..d {
        let mut tmp = q.get_coeff_at(i);
        if i & 1 != 0 {
            tmp = TFr::negate(&mut tmp);
        }
        assert!(tmp.is_one());
    }
    q.destroy();
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

fn new_test_poly<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>(coeffs: &Vec<i32>, len: usize) -> TPoly {
    let mut p = TPoly::new(len).unwrap();

    for i in 0..len {
        let coeff: i32 = coeffs[i];
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

pub fn poly_test_div<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {

    for i in 0..7 {
        let divided_data = test_data(i, 0);
        let divisor_data = test_data(i, 1);
        let expected_data = test_data(i, 2);
        let mut dividend: TPoly = new_test_poly(&divided_data, divided_data.len());
        let divisor: TPoly = new_test_poly(&divisor_data, divisor_data.len());
        let expected: TPoly = new_test_poly(&expected_data, expected_data.len());

        let result = dividend.div(&divisor);

        assert!(result.is_ok());
        let actual = result.unwrap();

        assert_eq!(expected.len(), actual.len());
        for i in 0..actual.len() {
            assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
        }
    }
}

pub fn poly_div_by_zero<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    //Arrange
    let coeffs: Vec<i32> = vec![1,1];
    let mut dividend: TPoly = new_test_poly(&coeffs, 2);
    let divisor = TPoly::new(0).unwrap();

    //Act
    let result = dividend.div(&divisor);
    
    //Assert
    assert!(result.is_err());

    //After
    dividend.destroy();
}

pub fn poly_mul_direct_test<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
    let coeffs0: Vec<i32> = vec![3, 4];
    let mut multiplicand: TPoly = new_test_poly(&coeffs0, 2);

    let coeffs1: Vec<i32> = vec![6, -5, 3];
    let mut multiplier: TPoly = new_test_poly(&coeffs1, 3);

    let coeffs2: Vec<i32> = vec![18, 9, -11, 12];
    let expected: TPoly = new_test_poly(&coeffs2, 4);

    let result0 = multiplicand.mul_direct(&multiplier, 4);
    assert!(result0.is_ok());
    let actual = result0.unwrap();

    for i in 0..actual.len() {
        assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
    }

    //Check commutativity
    let result1 = multiplier.mul_direct(&multiplicand, 4);
    assert!(result1.is_ok());
    let actual = result1.unwrap();

    for i in 0..actual.len() {
        assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
    }

    multiplicand.destroy();
    multiplier.destroy();
}

pub fn poly_mul_fft_test<TFr: Fr,  TFTTSettings: FFTSettings<TFr>, TPoly: Poly<TFr, TFTTSettings>>() {
    let coeffs: Vec<i32> = vec![3, 4];
    let mut multiplicand: TPoly = new_test_poly(&coeffs, 2);

    let coeffs: Vec<i32> = vec![6, -5, 3];
    let mut multiplier: TPoly = new_test_poly(&coeffs, 3);

    let coeffs: Vec<i32> = vec![18, 9, -11, 12];
    let expected: TPoly = new_test_poly(&coeffs, 4);

    let result = multiplicand.mul_fft(&multiplier, 4, None);
    assert!(result.is_ok());
    let actual = result.unwrap();

    for i in 0..actual.len() {
        assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
    }

    //Check commutativity
    let result = multiplier.mul_fft(&multiplicand, 4, None);
    assert!(result.is_ok());
    let actual = result.unwrap();

    for i in 0..actual.len() {
        assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
    }

    multiplicand.destroy();
    multiplier.destroy();
}

// // NOT FINISHED, only to be used if there would be Direct and FFT multiplications
// pub fn poly_mul_random<TFr: Fr, TFTTSettings: FFTSettings<TFr>,TPoly: Poly<TFr, TFTTSettings>>() {
//     let mut rng = StdRng::seed_from_u64(0);
//     for k in 0..256 {
//         let multiplicand_length: usize = 1 + (rng.next_u64() % 1000);
//         let multiplier_length: usize = 1 + (rng.next_u64() % 1000);
//         let out_length: usize = 1 + (rng.next_u64() % 1000);

//         let multiplicand = TPoly::new(multiplicand_length).unwrap();
//         let multiplier = TPoly::new(multiplier_length).unwrap();

//         for i in 0..multiplicand_length {
//             let coef = TFr::rand();
//             multiplicand.set_coeff_at(i, &coef);
//         }

//         for i in 0..multiplier_length {
//             let coef = TFr::rand();
//             multiplier.set_coeff_at(i, &coef);
//         }

//         //Ensure that the polynomials' orders corresponds to their lengths
//         if multiplicand.get_coeff_at(multiplicand.len() - 1).is_zero() {
//             let fr_one = Fr::one();
//             multiplicand.set_coeff_at(multiplicand.len() - 1, &fr_one);
//         }

//         if multiplier.get_coeff_at(multiplier.len() - 1).is_zero() {
//             let fr_one = Fr::one();
//             multiplier.set_coeff_at(multiplier.len() - 1, &fr_one);
//         }

//         let q0 = TPoly::new(out_length);
//         let result0 = multiplicand.mul(&multiplier);
//         assert!(result0.is_ok());
//     }
// }
