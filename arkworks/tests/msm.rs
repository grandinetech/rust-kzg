#[cfg(test)]
mod tests {
    use kzg_bench::tests::msm::msm_slice::{
        test_msm_slice_window_size_1, test_msm_slice_window_size_16, test_msm_slice_window_size_2,
        test_msm_slice_window_size_3,
    };

    #[test]
    pub fn test_msm_slice_window_size_1_() {
        test_msm_slice_window_size_1()
    }

    #[test]
    fn test_msm_slice_window_size_2_() {
        test_msm_slice_window_size_2();
    }

    #[test]
    fn test_msm_slice_window_size_3_() {
        test_msm_slice_window_size_3();
    }

    #[test]
    fn test_msm_slice_window_size_16_() {
        test_msm_slice_window_size_16();
    }
}
