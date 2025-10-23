#![no_main]

use kzg::{Fr, G1Affine, G1};
use libfuzzer_sys::{
    arbitrary::{self, Arbitrary},
    fuzz_target, Corpus,
};
use rust_kzg_blst::types::{fr::FsFr, g1::FsG1Affine};
use rust_kzg_fuzz::{backends, compute_expected_value};
use std::fmt::{self, Debug, Formatter};

struct FuzzInput {
    points: Vec<[u8; 48]>,
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
        let points = u
            .arbitrary_iter::<FsG1Affine>()?
            .map(|p| p.map(|p| p.to_proj().to_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        let scalars = u
            .arbitrary_iter::<FsFr>()?
            .map(|s| s.map(|s| s.to_bytes()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { points, scalars })
    }
}

fuzz_target!(|input: FuzzInput| -> Corpus {
    if input.points.len() != input.scalars.len() {
        return Corpus::Reject;
    }

    let b = backends(&input.points);
    let expected = compute_expected_value(&input.points, &input.scalars);

    for backend in b {
        let received = backend.multiply(&input.scalars);

        if expected != received {
            backend
                .save_case(&input.points, &input.scalars)
                .expect("failed to save test case");
        }

        assert_eq!(expected, received, "backend: {}", backend.name());
    }

    Corpus::Keep
});
