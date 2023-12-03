mod pippenger;
#[cfg(all(feature = "parallel", feature = "std"))]
mod pippenger_parallel;

extern crate alloc;

use crate::types::{fr::FsFr, g1::FsG1};
use alloc::string::{String, ToString};
use kzg::G1Mul;

pub fn multi_scalar_multiplication(points: &[FsG1], scalars: &[FsFr]) -> Result<FsG1, String> {
    if points.len() != scalars.len() {
        return Err("Point and scalars length not match".to_string());
    }

    if points.len() == 1 {
        return Ok(points[0].mul(&scalars[0]));
    }

    Ok(pippenger::pippenger(points, scalars))
}
