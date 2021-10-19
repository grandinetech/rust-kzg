#[cfg(test)]
mod tests {
    use kzg_bench::poly::create_poly_of_length_ten;
    use kzg_bindings::finite::BlstFr;
    use kzg_bindings::poly::KzgPoly;

    #[test]
    fn test_create_poly_of_length_ten() {
        create_poly_of_length_ten::<BlstFr, KzgPoly>();
    }
}
