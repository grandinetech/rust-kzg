use mcl_rust::data_types::fr::Fr;
use mcl_rust::kzg10::*;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;
use std::vec;

#[test]
fn polynomial_new_works_with_valid_params() {
    // Arrange
    assert!(init(CurveType::BLS12_381));
    let coefficients = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    // Act
    // Assert
    let _poly = Polynomial::from_i32(&coefficients);
}

#[test]
fn polynomial_eval_at_should_specific_value_given_exact_inputs() {
    // Arrange
    assert!(init(CurveType::BLS12_381));
    let coefficients = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly = Polynomial::from_i32(&coefficients);
    // Act
    let value = poly.eval_at(&Fr::from_int(17));
    // Assert
    let expected = "39537218396363405614";
    let actual = value.get_str(10);
    assert_eq!(expected, actual);
}

#[test]
fn extend_poly_appends_fr_zero() {
    // Arrange
    assert!(init(CurveType::BLS12_381));
    let poly = Polynomial::from_i32(&[1, 2, 3, 4]);
    // Act
    let extended = poly.get_extended(8);

    // Assert
    let expected = vec!["1", "2", "3", "4", "0", "0", "0", "0"];
    for (i, item) in expected.iter().enumerate().take(8) {
        // for i in 0..8 {
        assert_eq!(*item, extended.coeffs[i].get_str(10));
    }
}

#[test]
fn poly_eval_0_check() {
    //Arrange
    assert!(init(CurveType::BLS12_381));
    let mut coefficients = Vec::new();
    let n: i32 = 7;
    let a: i32 = 597;

    for i in 0..n {
        coefficients.push(i + a);
    }
    let p = Polynomial::from_i32(&coefficients);
    let expected = Fr::from_int(a);

    //Act
    let actual = p.eval_at(&Fr::default());

    //Assert
    assert_eq!(expected, actual);
}

fn get_test_vec(first: usize, second: usize) -> Vec<Fr> {
    // (x^2 - 1) / (x + 1) = x - 1
    let vec_0_0: Vec<Fr> = vec![Fr::from_int(-1), Fr::from_int(0), Fr::from_int(1)];
    let vec_0_1: Vec<Fr> = vec![Fr::from_int(1), Fr::from_int(1)];
    let vec_0_2: Vec<Fr> = vec![Fr::from_int(-1), Fr::from_int(1)];

    // (12x^3 - 11x^2 + 9x + 18) / (4x + 3) = 3x^2 - 5x + 6
    let vec_1_0: Vec<Fr> = vec![
        Fr::from_int(18),
        Fr::from_int(9),
        Fr::from_int(-11),
        Fr::from_int(12),
    ];
    let vec_1_1: Vec<Fr> = vec![Fr::from_int(3), Fr::from_int(4)];
    let vec_1_2: Vec<Fr> = vec![Fr::from_int(6), Fr::from_int(-5), Fr::from_int(3)];

    // (x + 1) / (x^2 - 1) = nil !!DOES NOT WORK WITH CURRENT IMPLEMENTATION OF METHOD
    let vec_2_0: Vec<Fr> = vec![Fr::from_int(1), Fr::from_int(1)];
    let vec_2_1: Vec<Fr> = vec![Fr::from_int(-1), Fr::from_int(0), Fr::from_int(2)];
    let vec_2_2: Vec<Fr> = vec![];

    let data: [[Vec<Fr>; 3]; 3] = [
        [vec_0_0, vec_0_1, vec_0_2],
        [vec_1_0, vec_1_1, vec_1_2],
        [vec_2_0, vec_2_1, vec_2_2],
    ];

    data[first][second].clone()
}

#[test]
fn poly_test_long_division() {
    assert!(init(CurveType::BLS12_381));
    const SIZE: usize = 2; //should be <= size of test data in get_test_vec()

    for i in 0..SIZE {
        let first = get_test_vec(i, 0);
        let second = get_test_vec(i, 1);
        let third = get_test_vec(i, 2);

        let dividend = Polynomial::from_fr(first);
        let expected = Polynomial::from_fr(third);

        let result = dividend.long_division(&second).unwrap();

        let expected_len: usize = expected.coeffs.len();
        let result_len: usize = result.coeffs.len();

        assert_eq!(expected_len, result_len);
        for j in 1..result_len {
            assert_eq!(result.coeffs[j], expected.coeffs[j]);
        }
    }
}

/*Library could be improved to use status codes/Result<T, E> or something like that,
since in current implementation it simply panics*/
#[test]
fn poly_div_by_zero() {
    //Arrange
    assert!(init(CurveType::BLS12_381));
    let dividend_vec: Vec<i32> = vec![1, 1];
    let divisor: Vec<Fr> = vec![Fr::default()];
    let dividend = Polynomial::from_i32(&dividend_vec);

    //Act
    let dummy = dividend.long_division(&divisor);

    //Assert
    assert!(dummy.is_err());
}
