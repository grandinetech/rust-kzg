#[cfg(test)]
mod tests {
    use kzg_bench::tests::msm::{batch_adder::{test_phase_one_zero_or_neg, test_phase_one_p_add_p, test_phase_one_p_add_q, test_phase_one_p_add_q_twice, test_phase_two_zero_add_p, test_phase_two_p_add_neg, test_phase_two_p_add_q, test_phase_two_p_add_p, test_batch_add, test_batch_add_step_n, test_batch_add_indexed_single_bucket, test_batch_add_indexed}, bucket_msm::{test_process_point_and_slices_deal_two_points, test_process_point_and_slices_deal_three_points, test_process_point_and_slices_glv_deal_two_points}};
    use rust_kzg_arkworks::kzg_types::{ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine};

    #[test]
    fn test_process_point_and_slices_deal_two_points_() {
    }

    #[test]
    pub fn test_process_point_and_slices_deal_three_points_() {
        test_process_point_and_slices_deal_three_points::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>();
    }

    #[test]
    fn test_process_point_and_slices_glv_deal_two_points_() {
        test_process_point_and_slices_glv_deal_two_points::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>();
    }

}
