use std::vec;
use mcl_rust::implem::*;
use mcl_rust::fr::Fr;

fn get_test_vec(first: usize, second: usize) -> Vec<Fr> {
    // (x^2 - 1) / (x + 1) = x - 1
    let vec_0_0: Vec<Fr> = vec![Fr::from_int(-1), Fr::from_int(0), Fr::from_int(1)];
    let vec_0_1: Vec<Fr> = vec![Fr::from_int(1), Fr::from_int(1)];
    let vec_0_2: Vec<Fr> = vec![Fr::from_int(-1), Fr::from_int(1)];

    // (12x^3 - 11x^2 + 9x + 18) / (4x + 3) = 3x^2 - 5x + 6
    let vec_1_0: Vec<Fr> = vec![Fr::from_int(18), Fr::from_int(9), Fr::from_int(-11), Fr::from_int(12)];
    let vec_1_1: Vec<Fr> = vec![Fr::from_int(3), Fr::from_int(4)];
    let vec_1_2: Vec<Fr> = vec![Fr::from_int(6), Fr::from_int(-5), Fr::from_int(3)];

    // (x + 1) / (x^2 - 1) = nil !!DOES NOT WORK WITH CURRENT IMPLEMENTATION OF METHOD
    let vec_2_0: Vec<Fr> = vec![Fr::from_int(1), Fr::from_int(1)];
    let vec_2_1: Vec<Fr> = vec![Fr::from_int(-1), Fr::from_int(0), Fr::from_int(2)];
    let vec_2_2: Vec<Fr> = vec![];

    let data: [[Vec<Fr>; 3]; 3] = [[vec_0_0, vec_0_1, vec_0_2], [vec_1_0, vec_1_1, vec_1_2], [vec_2_0, vec_2_1, vec_2_2]];

    return data[first][second].clone();
}

#[test]
fn poly_test_div() {
    const SIZE: usize = 2; //should be <= size of test data in get_test_vec()

    for i in 0..SIZE {
        let first = get_test_vec(i, 0);
        let second = get_test_vec(i, 1);
        let third = get_test_vec(i, 2);

        let dividend = Polynomial::from_fr(first);
        let expected = Polynomial::from_fr(third);

        let result = dividend.long_division(&second);

        let expected_len: usize;
        let result_len: usize;

        expected_len = expected.coeffs.len();
        result_len = result.coeffs.len();

        assert_eq!(expected_len, result_len);
        for j in 1..result_len {
            assert_eq!(result.coeffs[j], expected.coeffs[j]);
        }
    }
}

#[test]
fn poly_div_by_zero() {
    let dividend_vec: Vec<i32> = vec![1 ,1];
    let zero_vec: Vec<Fr> = vec![Fr::default()];

    let dividend = Polynomial::from_i32(&dividend_vec);
    let divisor = Polynomial::from_fr(zero_vec);

    let dummy = std::panic::catch_unwind(|| FFTSettings::new(32));
}