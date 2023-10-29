#[cfg(test)]
pub mod tests {
    use kzg::common_utils::reverse_bit_order;

    #[test]
    fn reverse_bit_order_bad_arguments() {
        // empty array should fail
        assert!(reverse_bit_order(&mut [0u8; 0]).is_err());
        // array with 1 element should be ignored
        assert!(reverse_bit_order(&mut [1u8]).is_ok());
        // array with 3 elements should fail, because 3 is not a power of 2
        assert!(reverse_bit_order(&mut [1u8, 2u8, 3u8]).is_err());
        // array with 4 elements should pass
        assert!(reverse_bit_order(&mut [1u8, 2u8, 3u8, 4u8]).is_ok());
    }
}
