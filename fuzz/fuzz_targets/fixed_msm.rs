#![no_main]

use kzg::{Fr, G1Mul, G1};
use libfuzzer_sys::{
    arbitrary::{self, Arbitrary},
    fuzz_target,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rust_kzg_blst::types::{fr::FsFr, g1::FsG1};
use rust_kzg_fuzz::{cached_backends, compute_expected_value, get_npoints, seed};
use std::{
    fmt::{self, Debug, Formatter},
    sync::OnceLock,
};

static POINTS: OnceLock<Vec<[u8; 48]>> = OnceLock::new();

fn generate_points(seed: <ChaCha20Rng as SeedableRng>::Seed) -> Vec<[u8; 48]> {
    let npoints = get_npoints();

    let mut rng = ChaCha20Rng::from_seed(seed);

    (0..npoints)
        .map(|_| {
            let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();
            let p = FsG1::generator().mul(&fr);
            p.to_bytes()
        })
        .collect::<Vec<_>>()
}

struct FuzzInput {
    scalars: Vec<[u8; 32]>,
}

impl Debug for FuzzInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FuzzInput")
            .field(&"Failing test case was saved to a file in current working directory")
            .finish()
    }
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let npoints = get_npoints();

        let mut scalars = Vec::new();
        for _ in 0..npoints {
            scalars.push(u.arbitrary::<FsFr>()?.to_bytes());
        }

        Ok(Self { scalars })
    }
}

fuzz_target!(|input: FuzzInput| {
    let points = POINTS.get_or_init(|| generate_points(seed()));
    let backends = cached_backends(&points);

    let expected = compute_expected_value(points, &input.scalars);

    for backend in backends {
        let received = backend.multiply(&input.scalars);

        if expected != received {
            backend
                .save_case(&points, &input.scalars)
                .expect("failed to save test case");
        }

        assert_eq!(expected, received, "backend: {}", backend.name());
    }
});
