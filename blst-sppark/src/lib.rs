// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use blst::*;
use std::ffi::c_void;

pub fn prepare_msm(points: &[blst_p1_affine]) -> *mut c_void {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn prepare_msm(points: *const blst_p1_affine, npoints: usize) -> *mut c_void;
    }

    let npoints = points.len();

    unsafe { prepare_msm(&points[0], npoints) }
}

pub fn mult_pippenger_prepared(msm: *mut c_void, scalars: &[blst_fr]) -> blst_p1 {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn mult_prepared_pippenger(
            msm: *mut c_void,
            out: *mut blst_p1,
            npoints: usize,
            scalars: *const blst_scalar,
        ) -> sppark::Error;
    }

    let npoints = scalars.len();
    let mut ret = blst_p1::default();

    unsafe { mult_prepared_pippenger(msm, &mut ret, npoints, &scalars[0]) }
}

pub fn mult_pippenger(points: &[blst_p1_affine], scalars: &[blst_fr]) -> blst_p1 {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn mult_pippenger(
            out: *mut blst_p1,
            points: *const blst_p1_affine,
            npoints: usize,
            scalars: *const blst_fr,
        ) -> sppark::Error;
    }

    let npoints = points.len();
    if npoints != scalars.len() {
        panic!("length mismatch")
    }

    let mut ret = blst_p1::default();
    let err = unsafe { mult_pippenger(&mut ret, &points[0], npoints, &scalars[0]) };
    if err.code != 0 {
        panic!("{}", String::from(err));
    }
    ret
}
