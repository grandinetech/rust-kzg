use kzg::{msm::arkmsm::arkmsm_msm::VariableBaseMSM, Scalar256};

pub fn test_msm_slice_window_size_1() {
    let scalar = Scalar256::from_u64_s(0b101);
    let mut slices: Vec<u32> = vec![0; 3];
    VariableBaseMSM::msm_slice(scalar, &mut slices, 1);
    assert!(slices.iter().eq([1, 0, 1].iter()));
}

pub fn test_msm_slice_window_size_2() {
    let scalar = Scalar256::from_u64_s(0b000110);
    let mut slices: Vec<u32> = vec![0; 3];
    VariableBaseMSM::msm_slice(scalar, &mut slices, 2);
    assert!(slices.iter().eq([2, 1, 0].iter()));
}

pub fn test_msm_slice_window_size_3() {
    let scalar = Scalar256::from_u64_s(0b010111000);
    let mut slices: Vec<u32> = vec![0; 3];
    VariableBaseMSM::msm_slice(scalar, &mut slices, 3);
    assert!(slices.iter().eq([0, 0x80000001, 3].iter()));
}

pub fn test_msm_slice_window_size_16() {
    let scalar = Scalar256::from_u64_s(0x123400007FFF);
    let mut slices: Vec<u32> = vec![0; 3];
    VariableBaseMSM::msm_slice(scalar, &mut slices, 16);
    assert!(slices.iter().eq([0x7FFF, 0, 0x1234].iter()));
}
