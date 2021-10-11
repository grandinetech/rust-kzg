#[cfg(test)]
mod tests {
    use kzg::Fr;

    #[test]
    fn sum_of_two_zeros_is_zero() {
        let zero = Fr::default();
        assert_eq!(kzg::finite::add_fr(zero, zero), zero);
    }
}
