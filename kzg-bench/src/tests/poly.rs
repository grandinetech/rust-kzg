use kzg::{Fr, Poly};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

pub fn create_poly_of_length_ten<TFr: Fr, TPoly: Poly<TFr>>() {
    let mut poly = TPoly::new(10).unwrap();
    assert_eq!(poly.len(), 10);
    poly.destroy();
}

pub fn poly_eval_check<TFr: Fr, TPoly: Poly<TFr>>() {
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

pub fn poly_eval_0_check<TFr: Fr, TPoly: Poly<TFr>>() {
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

pub fn poly_eval_nil_check<TFr: Fr, TPoly: Poly<TFr>>() {
    let n: usize = 0;
    let mut poly = TPoly::new(n).unwrap();
    let actual = poly.eval(&TFr::one());
    assert!(actual.equals(&TFr::zero()));
    poly.destroy();
}

pub fn poly_inverse_simple_0<TFr: Fr, TPoly: Poly<TFr>>() {
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

pub fn poly_inverse_simple_1<TFr: Fr, TPoly: Poly<TFr>>() {
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

fn new_test_poly<TFr: Fr, TPoly: Poly<TFr>>(coeffs: &[i32], len: usize) -> TPoly {
    let mut p = TPoly::new(len).unwrap();

    for i in 0..len {
        let coeff: i32 = coeffs[i];
        if coeff >= 0 {
            let c = TFr::from_u64(coeff as u64);
            p.set_coeff_at(i, &c);
        }
        else {
            let c = TFr::from_u64((-coeff) as u64);
            let negc = c.negate();
            p.set_coeff_at(i, &negc);
        }
    }

    p
}

pub fn poly_test_div<TFr: Fr, TPoly: Poly<TFr>>() {

    //Should be improved with more test data as in C version
    let coeffs: [i32; 3] = [-1, 0, 1];
    let mut dividend: TPoly = new_test_poly(&coeffs, 3);

    let coeffs: [i32; 2] = [1, 1];
    let divisor: TPoly = new_test_poly(&coeffs, 2);

    let coeffs: [i32; 2] = [-1, 1];
    let expected: TPoly = new_test_poly(&coeffs, 2);

    let result = dividend.div(&divisor);
    assert!(result.is_ok());
    let actual = result.unwrap();

    assert_eq!(expected.len(), actual.len());
    for i in 0..actual.len() {
        assert!(expected.get_coeff_at(i).equals(&actual.get_coeff_at(i)))
    }

    dividend.destroy();

}

pub fn poly_mul_direct_test<TFr: Fr, TPoly: Poly<TFr>>() {
    let coeffs0: [i32; 2] = [3, 4];
    let mut multiplicand: TPoly = new_test_poly(&coeffs0, 2);

    let coeffs1: [i32; 3] = [6, -5, 3];
    let mut multiplier: TPoly = new_test_poly(&coeffs1, 3);

    let coeffs2: [i32; 4] = [18, 9, -11, 12];
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

//NOT FINISHED, only to be used if there would be Direct and FFT multiplications
// pub fn poly_mul_random<TFr: Fr, TPoly: Poly<TFr>>() {
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
//         if (multiplicand.get_coeff_at(multiplicand.len() - 1).is_zero()) {
//             let fr_one = Fr::one();
//             multiplicand.set_coeff_at(multiplicand.len() - 1, &fr_one);
//         }

//         if (multiplier.get_coeff_at(multiplier.len() - 1).is_zero()) {
//             let fr_one = Fr::one();
//             multiplier.set_coeff_at(multiplier.len() - 1, &fr_one);
//         }

//         let q0 = TPoly::new(out_length);
//         let result0 = multiplicand.mul(&multiplier);
//         assert!(result0.is_ok());
//     }
// }
