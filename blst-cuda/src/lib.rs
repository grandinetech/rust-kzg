// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use blst::*;

pub fn multi_scalar_mult(
    points: &[blst_p1_affine],
    scalars: &[blst_scalar],
) -> blst_p1 {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn mult_pippenger(
            out: *mut blst_p1,
            points: *const blst_p1_affine,
            npoints: usize,
            scalars: *const blst_scalar,
        ) -> sppark::Error;
    }

    let npoints = points.len();
    if npoints != scalars.len() {
        panic!("length mismatch")
    }

    let mut ret = blst_p1::default();
    let err =
        unsafe { mult_pippenger(&mut ret, &points[0], npoints, &scalars[0]) };
    if err.code != 0 {
        panic!("{}", String::from(err));
    }
    ret
}

pub fn prepare_multi_scalar_mult(
    points: &[blst_p1_affine],
) {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn prepare_msm(
            points: *const blst_p1_affine,
            npoints: usize,
        );
    }

    let npoints = points.len();

    unsafe { prepare_msm(&points[0], npoints) };  
}

pub fn run_prepared_multi_scalar_mult(
    scalars: &[blst_scalar],
) {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn mult_prepared_pippenger(
            out: *mut blst_p1,
            npoints: usize,
            scalars: *const blst_scalar,
        ) -> sppark::Error;
    }

    let npoints = scalars.len();
    let mut ret = blst_p1::default();

    unsafe { mult_prepared_pippenger(&mut ret, npoints, &scalars[0]) };  
}