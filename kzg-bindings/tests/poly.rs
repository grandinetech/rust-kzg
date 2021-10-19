#[cfg(test)]
mod tests {
    use kzg_bench::poly::{create_poly_of_length_ten, poly_eval_check, poly_eval_0_check,
                          poly_eval_nil_check, poly_inverse_simple_0, poly_inverse_simple_1};
    use kzg_bindings::finite::BlstFr;
    use kzg_bindings::poly::KzgPoly;

    #[test]
    fn test_create_poly_of_length_ten() {
        create_poly_of_length_ten::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_eval_check() {
        poly_eval_check::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_eval_0_check() {
        poly_eval_0_check::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_eval_nil_check() {
        poly_eval_nil_check::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_inverse_simple_0() {
        poly_inverse_simple_0::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_inverse_simple_1() {
        poly_inverse_simple_1::<BlstFr, KzgPoly>();
    }
}
