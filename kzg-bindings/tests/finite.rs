#[cfg(test)]
mod tests {
    use kzg_bench::tests::finite::sum_of_two_zeros_is_zero;
    use kzg_bindings::finite::BlstFr;

    #[test]
    fn test_sum_of_two_zeros_is_zero() {
        sum_of_two_zeros_is_zero::<BlstFr>();
    }
}
