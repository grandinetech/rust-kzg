#[cfg(test)]
mod tests {
    use kzg::Poly;

    #[test]
    fn create_poly_of_length_ten() {
        let mut poly = match kzg::poly::create_poly(10) {
            Ok(p) => p,
            Err(_) => Poly::default()
        };
        assert_eq!(poly.length, 10);
        kzg::poly::destroy_poly(&mut poly);
    }
}
