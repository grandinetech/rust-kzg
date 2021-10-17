#[cfg(test)]
mod tests {
    use kzg::{Fr, Poly};

    #[test]
    fn create_poly_of_length_ten() {
        let mut poly = Poly::new(10).unwrap();
        assert_eq!(poly.length, 10);
        poly.destroy();
    }

    #[test]
    fn poly_eval_check() {
        let n: u64 = 10;
        let mut poly = Poly::new(n).unwrap();
        for i in 0..n {
            let fr = Fr::from_u64(i + 1);
            poly.set_coeff_at(i, fr);
        }
        let expected = Fr::from_u64(n * (n + 1) / 2);
        let actual = poly.eval_at(&Fr::one());
        assert!(Fr::is_equal(expected, actual));
        poly.destroy();
    }

    #[test]
    fn poly_eval_0_check() {
        let n: u64 = 7;
        let a: u64 = 597;
        let mut poly = Poly::new(n).unwrap();
        for i in 0..n {
            let fr = Fr::from_u64(i + a);
            poly.set_coeff_at(i, fr);
        }
        let expected = Fr::from_u64(a);
        let actual = poly.eval_at(&Fr::zero());
        assert!(Fr::is_equal(expected, actual));
        poly.destroy();
    }

    #[test]
    fn poly_eval_nil_check() {
        let n: u64 = 0;
        let mut poly = Poly::new(n).unwrap();
        let actual = poly.eval_at(&Fr::one());
        assert!(Fr::is_equal(Fr::zero(), actual));
        poly.destroy();
    }

    #[test]
    fn poly_inverse_simple_0() {
        // 1 / (1 - x) = 1 + x + x^2 + ...
        let d: u64 = 16;
        let mut p = Poly::new(2).unwrap();
        p.set_coeff_at(0, Fr::one());
        p.set_coeff_at(1, Fr::one());
        p.set_coeff_at(1, Fr::negate(&p.get_coeff_at(1)));
        let mut q = Poly::new(d).unwrap();
        let result = q.inverse(&mut p);
        assert!(result.is_ok());
        for i in 0..d {
            assert!(q.get_coeff_at(i).is_one());
        }
        p.destroy();
        q.destroy();
    }

    #[test]
    fn poly_inverse_simple_1() {
        // 1 / (1 + x) = 1 - x + x^2 - ...
        let d: u64 = 16;
        let mut p = Poly::new(2).unwrap();
        p.set_coeff_at(0, Fr::one());
        p.set_coeff_at(1, Fr::one());
        let mut q = Poly::new(d).unwrap();
        let result = q.inverse(&mut p);
        assert!(result.is_ok());
        for i in 0..d {
            let mut tmp = q.get_coeff_at(i);
            if i & 1 != 0 {
                tmp = Fr::negate(&mut tmp);
            }
            assert!(tmp.is_one());
        }
        p.destroy();
        q.destroy();
    }
}
