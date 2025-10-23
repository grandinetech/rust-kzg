#![no_main]

use libfuzzer_sys::{arbitrary::{self, Arbitrary}, fuzz_target};
use std::{sync::OnceLock, env};
use kzg::{msm::{msm_impls::{pippenger, msm}, precompute::{precompute, PrecomputationTable}}, Fr, G1, G1Mul};
use rand::{seq::SliceRandom, Rng};
use rust_kzg_blst::types::{fp::FsFp, fr::FsFr, g1::{FsG1, FsG1Affine, FsG1ProjAddAffine}};

static TABLE: OnceLock<Option<PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>>> = OnceLock::new();
static NPOINTS: OnceLock<usize> = OnceLock::new();
static POINTS: OnceLock<Vec<FsG1>> = OnceLock::new();

fn get_npoints() -> usize {
    NPOINTS.get_or_init(|| {
        let npow: usize = env::var("NPOW").unwrap_or("12".to_owned()).parse().unwrap();
        let npoints: usize = 1usize << npow;
        npoints
    }).clone()
}

fn init() {
    let npoints = get_npoints();

    let mut rng = rand::thread_rng();

    // random amount of infinity points, between 50
    let mut numzero = rng.gen_range((npoints / 2)..npoints);

    let mut points = (0..numzero).map(|_| FsG1::zero()).chain((numzero..npoints).map(|_| {
        let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();
        let p = FsG1::generator().mul(&fr);
        p
    })).collect::<Vec<_>>();

    points.shuffle(&mut rng);

    let table = precompute(&points, &[]).unwrap();

    TABLE.set(table).unwrap();
    POINTS.set(points).unwrap();
}

#[derive(Debug)]
struct FuzzInput {
    scalars: Vec<FsFr>
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let npoints = get_npoints();

        let mut scalars = Vec::new();
        for _ in 0..npoints {
            if u.arbitrary()? {
                scalars.push(u.arbitrary()?);
            } else {
                scalars.push(FsFr::zero());
            }
        }

        Ok(Self { scalars })
    }
}

fuzz_target!(
    init: {
        init();
    },
    |input: FuzzInput| {
        let points = POINTS.get().unwrap();
        let expected = pippenger::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(points, &input.scalars);
        let received = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(points, &input.scalars, get_npoints(), TABLE.get().unwrap().as_ref());

        assert_eq!(expected, received)
    }
);
