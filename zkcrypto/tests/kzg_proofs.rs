#[cfg(test)]
pub mod tests {
	use kzg_bench::tests::kzg_proofs::*;
	use zkcrypto::poly::{ZPoly};
	use zkcrypto::zkfr::blsScalar;
	use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
	use zkcrypto::fftsettings::ZkFFTSettings;
	use zkcrypto::kzg_proofs::{KZGSettings, generate_trusted_setup};

	#[test]
	fn test_proof_single() {
		proof_single::<blsScalar, ZkG1Projective, ZkG2Projective, ZPoly, ZkFFTSettings, KZGSettings>(&generate_trusted_setup);
	}

	// #[test]
	// fn test_commit_to_nil_poly() {

	// }
}