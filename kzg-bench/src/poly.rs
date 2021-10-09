#[cfg(test)]
mod tests {
    use kzg::{Error, Poly};

    #[test]
    fn create_poly_of_length_ten() {
        let mut poly = match kzg::poly::create_poly(10) {
            Ok(p) => p,
            Err(e) => {
                println!("Poly error: {:?}", e);
                Poly::default()
            }
        };
        assert_eq!(poly.length, 10);
        kzg::poly::destroy_poly(&mut poly);
    }

    #[test]
    fn create_divided_poly_in_finite_field() {
        let errors = kzg::poly::poly_division_in_finite_field(15);
        assert_eq!(errors, Error::KzgOk);
    }

    #[test]
    fn poly_eval_check() {
        let n: u64 = 10;
        let mut poly = match kzg::poly::create_poly(n) {
            Ok(p) => p,
            Err(e) => {
                println!("Poly error: {:?}", e);
                Poly::default()
            }
        };
        for i in 0..n {
            let fr = kzg::finite::u64_to_fr(i + 1);
            kzg::poly::change_poly_coeff(&mut poly, i as isize, fr);
        }
        let expected = kzg::finite::u64_to_fr(n * (n + 1) / 2);
        let actual = kzg::poly::eval_poly_at(&mut poly, &kzg::finite::one_fr());
        assert_eq!(kzg::finite::is_equal(expected, actual), true);
        kzg::poly::destroy_poly(&mut poly);
    }

    #[test]
    fn poly_eval_0_check() {
        let n: u64 = 7;
        let a: u64 = 597;
        let mut poly = match kzg::poly::create_poly(n) {
            Ok(p) => p,
            Err(e) => {
                println!("Poly error: {:?}", e);
                Poly::default()
            }
        };
        for i in 0..n {
            let fr = kzg::finite::u64_to_fr(i + a);
            kzg::poly::change_poly_coeff(&mut poly, i as isize, fr);
        }
        let expected = kzg::finite::u64_to_fr(a);
        let actual = kzg::poly::eval_poly_at(&mut poly, &kzg::finite::zero_fr());
        assert_eq!(kzg::finite::is_equal(expected, actual), true);
        kzg::poly::destroy_poly(&mut poly);
    }

    #[test]
    fn poly_eval_nil_check() {
        let n: u64 = 0;
        let mut poly = match kzg::poly::create_poly(n) {
            Ok(p) => p,
            Err(e) => {
                println!("Poly error: {:?}", e);
                Poly::default()
            }
        };
        let actual = kzg::poly::eval_poly_at(&mut poly, &kzg::finite::one_fr());
        assert_eq!(kzg::finite::is_equal(kzg::finite::zero_fr(), actual), true);
        kzg::poly::destroy_poly(&mut poly);
    }
}
