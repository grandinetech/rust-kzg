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
}
