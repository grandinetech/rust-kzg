#[cfg(test)]
mod tests {
    use kzg_bench::tests::msm::batch_adder::{
        test_batch_add, test_batch_add_indexed, test_batch_add_indexed_single_bucket,
        test_batch_add_step_n, test_phase_one_p_add_p, test_phase_one_p_add_q,
        test_phase_one_p_add_q_twice, test_phase_one_zero_or_neg, test_phase_two_p_add_neg,
        test_phase_two_p_add_p, test_phase_two_p_add_q, test_phase_two_zero_add_p,
    };
    use rust_kzg_constantine::types::{
        fp::CtFp,
        g1::{CtG1, CtG1Affine},
    };
    // use rust_kzg_constantine::types::

    #[test]
    fn test_phase_one_zero_or_neg_() {
        test_phase_one_zero_or_neg::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_one_p_add_p_() {
        test_phase_one_p_add_p::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_one_p_add_q_() {
        test_phase_one_p_add_q::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_one_p_add_q_twice_() {
        test_phase_one_p_add_q_twice::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_two_zero_add_p_() {
        test_phase_two_zero_add_p::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_two_p_add_neg_() {
        test_phase_two_p_add_neg::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_two_p_add_q_() {
        test_phase_two_p_add_q::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_phase_two_p_add_p_() {
        test_phase_two_p_add_p::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_batch_add_() {
        test_batch_add::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_batch_add_step_n_() {
        test_batch_add_step_n::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_batch_add_indexed_() {
        test_batch_add_indexed::<CtG1, CtFp, CtG1Affine>();
    }

    #[test]
    fn test_batch_add_indexed_single_bucket_() {
        test_batch_add_indexed_single_bucket::<CtG1, CtFp, CtG1Affine>();
    }
}
