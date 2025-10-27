#![no_main]

use libfuzzer_sys::{arbitrary::{self, Arbitrary}, fuzz_target};
use std::{sync::OnceLock, env};
use kzg::{msm::{msm_impls::{pippenger, msm}, precompute::{precompute, PrecomputationTable}}, Fr, G1, G1Mul};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rust_kzg_blst::types::{fp::FsFp, fr::FsFr, g1::{FsG1, FsG1Affine, FsG1ProjAddAffine}};
use rand_chacha::ChaCha20Rng;

static TABLE: OnceLock<Option<PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>>> = OnceLock::new();
static NPOINTS: OnceLock<usize> = OnceLock::new();
static POINTS: OnceLock<Vec<FsG1>> = OnceLock::new();
static SEED: OnceLock<<ChaCha20Rng as SeedableRng>::Seed> = OnceLock::new();

fn get_npoints() -> usize {
    NPOINTS.get_or_init(|| {
        let npow: usize = env::var("NPOW").unwrap_or("12".to_owned()).parse().unwrap();
        let npoints: usize = 1usize << npow;
        npoints
    }).clone()
}

#[derive(Debug)]
struct FuzzInput {
    scalars: Vec<FsFr>,
    seed: <ChaCha20Rng as SeedableRng>::Seed
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let npoints = get_npoints();

        let mut scalars = Vec::new();
        for _ in 0..npoints {
            scalars.push(u.arbitrary()?);
        }

        Ok(Self { scalars, seed: SEED.get_or_init(|| rand::thread_rng().gen()).clone() })
    }
}

fuzz_target!(|input: FuzzInput| {
    let points = POINTS.get_or_init(|| {
        let npoints = get_npoints();

        let mut rng = ChaCha20Rng::from_seed(input.seed);

        let mut points = (0..npoints).map(|_| {
            let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();
            let p = FsG1::generator().mul(&fr);
            p
        }).collect::<Vec<_>>();

        points.shuffle(&mut rng);

        points
    });
    let table = TABLE.get_or_init(|| {
        precompute(&points, &[]).unwrap()
    });
    let expected = pippenger::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(points, &input.scalars);
    let received = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(points, &input.scalars, get_npoints(), TABLE.get().unwrap().as_ref());

    assert_eq!(expected, received)
});
