#[cfg(test)]
mod tests {
    use kzg_bench::tests::msm::bucket_msm::{
        test_process_point_and_slices_deal_three_points,
        test_process_point_and_slices_glv_deal_two_points,
    };
    use rust_kzg_arkworks4::kzg_types::{ArkFp, ArkG1, ArkG1Affine, ArkG1ProjAddAffine};

    #[test]
    pub fn test_process_point_and_slices_deal_three_points_() {
        test_process_point_and_slices_deal_three_points::<
            ArkG1,
            ArkFp,
            ArkG1Affine,
            ArkG1ProjAddAffine,
        >();
    }

    #[test]
    fn test_process_point_and_slices_glv_deal_two_points_() {
        test_process_point_and_slices_glv_deal_two_points::<
            ArkG1,
            ArkFp,
            ArkG1Affine,
            ArkG1ProjAddAffine,
        >();
    }
}
