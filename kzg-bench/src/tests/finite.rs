use kzg::Fr;

pub fn sum_of_two_zeros_is_zero<TFr: Fr>() {
    let zero = TFr::default();
    assert!(zero.add(&zero).equals(&zero));
}
