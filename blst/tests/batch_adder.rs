#[cfg(test)]
mod tests {
    use kzg_bench::tests::msm::batch_adder::{test_phase_one_zero_or_neg, test_phase_one_p_add_p, test_phase_one_p_add_q, test_phase_one_p_add_q_twice, test_phase_two_zero_add_p, test_phase_two_p_add_neg, test_phase_two_p_add_q, test_phase_two_p_add_p, test_batch_add, test_batch_add_step_n, test_batch_add_indexed_single_bucket, test_batch_add_indexed};
    use rust_kzg_blst::types::{g1::{FsG1Affine, FsG1}, fp::FsFp};
    // use rust_kzg_blst::types::

    #[test]
    fn test_phase_one_zero_or_neg_() {
        test_phase_one_zero_or_neg::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_one_p_add_p_() {
        test_phase_one_p_add_p::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_one_p_add_q_() {
        test_phase_one_p_add_q::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_one_p_add_q_twice_() {
        test_phase_one_p_add_q_twice::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_two_zero_add_p_() {
        test_phase_two_zero_add_p::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_two_p_add_neg_() {
        test_phase_two_p_add_neg::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_two_p_add_q_() {
        test_phase_two_p_add_q::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_phase_two_p_add_p_() {
        test_phase_two_p_add_p::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_batch_add_() {
        test_batch_add::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_batch_add_step_n_() {
        test_batch_add_step_n::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_batch_add_indexed_() {
        test_batch_add_indexed::<FsG1, FsFp, FsG1Affine>();
    }

    #[test]
    fn test_batch_add_indexed_single_bucket_() {
        test_batch_add_indexed_single_bucket::<FsG1, FsFp, FsG1Affine>();
    }
}
