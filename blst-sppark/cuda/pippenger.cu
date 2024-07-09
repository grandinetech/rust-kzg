// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#include <cuda.h>

#include <ff/bls12-381.hpp>

#include <ec/jacobian_t.hpp>
#include <ec/xyzz_t.hpp>

typedef jacobian_t<fp_t> point_t;
typedef xyzz_t<fp_t> bucket_t;
typedef bucket_t::affine_t affine_t;
typedef fr_t scalar_t;

#include <msm/pippenger.cuh>

#ifndef __CUDA_ARCH__

extern "C"
void *prepare_msm(const affine_t points[], size_t npoints) {
    return new msm_t<bucket_t, point_t, affine_t, scalar_t>{points, npoints};
}

extern "C"
RustError mult_pippenger_prepared(void *msm, point_t* out, size_t npoints, const scalar_t scalars[]) {
    return static_cast<msm_t<bucket_t, point_t, affine_t, scalar_t>*>(msm)->invoke(*out, slice_t<scalar_t>{scalars, npoints}, true);
}

extern "C"
RustError mult_pippenger(point_t* out, const affine_t points[], size_t npoints,
                                       const scalar_t scalars[])
{
    return mult_pippenger<bucket_t>(out, points, npoints, scalars, true);
}
#endif