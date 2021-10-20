use kzg::{Fr, Poly};

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
