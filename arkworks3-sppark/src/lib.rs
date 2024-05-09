// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use ark_bls12_381::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::AffineCurve;
use ark_ff::PrimeField;
use ark_std::Zero;

use std::ffi::c_void;

pub fn prepare_multi_scalar_mult<G: AffineCurve>(points: &[G]) -> *mut c_void {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn prepare_msm(points: *const G1Affine, npoints: usize, ffi_affine_sz: usize) -> *mut c_void;
    }

    let npoints = points.len();

    unsafe { prepare_msm(points.as_ptr() as *const _, npoints, std::mem::size_of::<G>()) }
}

pub fn multi_scalar_mult_prepared<G: AffineCurve>(msm: *mut c_void, scalars: &[<G::ScalarField as PrimeField>::BigInt]) -> G::Projective {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn mult_pippenger_prepared(
            msm: *mut c_void,
            out: *mut G1Projective,
            npoints: usize,
            scalars: *const Fr,
        ) -> sppark::Error;
    }

    let npoints = scalars.len();
    let mut ret = G::Projective::zero();

    let err = unsafe { mult_pippenger_prepared(msm, &mut ret as *mut _ as *mut _, npoints, scalars.as_ptr() as *const _) };
    if err.code != 0 {
        panic!("{}", String::from(err));
    }
    ret
}

pub fn multi_scalar_mult<G: AffineCurve>(points: &[G], scalars: &[<G::ScalarField as PrimeField>::BigInt]) -> G::Projective {
    #[cfg_attr(feature = "quiet", allow(improper_ctypes))]
    extern "C" {
        fn mult_pippenger(
            out: *mut G1Projective,
            points: *const G1Affine,
            npoints: usize,
            scalars: *const Fr,
            ffi_affine_sz: usize,
        ) -> sppark::Error;
    }

    let npoints = points.len();
    if npoints != scalars.len() {
        panic!("length mismatch")
    }

    let mut ret = G::Projective::zero();
    let err = unsafe { mult_pippenger(&mut ret as *mut _ as *mut _, points.as_ptr() as *const _, npoints, scalars.as_ptr() as *const _, std::mem::size_of::<G>()) };
    if err.code != 0 {
        panic!("{}", String::from(err));
    }
    ret
}
