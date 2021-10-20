#[cfg(test)]
mod tests {
    use std::vec;
    use mcl_rust::old::*;
    use mcl_rust::kzg10::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::CurveType;
    use mcl_rust::mcl_methods::init;

    #[test]
    fn polynomial_new_works_with_valid_params_INSAMEFOLDER() {
        // Arrange
        assert!(init(CurveType::BLS12_381));
        let coefficients = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
        
        // Act
        // Assert
        let _poly = Polynomial::from_i32(&coefficients);
    }

}
