#[cfg(test)]
pub mod tests {
	use kzg_bench::tests::consts::*;
    use zkcrypto::fftsettings::{ZkFFTSettings};
	use zkcrypto::zkfr::blsScalar;
	use zkcrypto::consts::{SCALE2_ROOT_OF_UNITY, expand_root_of_unity};
	
	
#[test]
fn roots_of_unity_out_of_bounds_fails_() {
    roots_of_unity_out_of_bounds_fails::<blsScalar, ZkFFTSettings>();
}

#[test]
fn roots_of_unity_are_plausible_() {
    roots_of_unity_are_plausible::<blsScalar>(&SCALE2_ROOT_OF_UNITY);
}

#[test]
fn expand_roots_is_plausible_() {
	expand_roots_is_plausible::<blsScalar>(&SCALE2_ROOT_OF_UNITY,
	&expand_root_of_unity);
}

#[test]
fn new_fft_settings_is_plausible_() {
	new_fft_settings_is_plausible::<blsScalar, ZkFFTSettings>();
}

}