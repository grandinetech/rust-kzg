use criterion::{criterion_group, criterion_main, Criterion};
use rust_kzg_blst::types::fp::FsFp;
use rust_kzg_blst::types::fr::FsFr;
use rust_kzg_blst::types::g1::{FsG1, FsG1Affine};
use kzg::Fr;
use kzg::G1;
use blst::{blst_p1_affine, blst_scalar_from_fr, blst_scalar};

extern crate alloc;

fn bench_g1_lincomb(c: &mut Criterion) {
    const NUM_POINTS: usize = 4096;

    let points = [FsG1::rand(); NUM_POINTS];
    let scalars = [FsFr::rand(); NUM_POINTS];

    let affines = kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(&points);
    let scalars = scalars.iter().map(|it| {
        let mut scalar = blst_scalar::default();

        unsafe { blst_scalar_from_fr(&mut scalar, &it.0) };

        scalar
    }).collect::<Vec<_>>();

    let affines = unsafe { alloc::slice::from_raw_parts(affines.as_ptr() as *const blst_p1_affine, affines.len()) };

    let id = format!("bench_g1_lincomb points: '{}'", NUM_POINTS);
    c.bench_function(&id, |b| {
        b.iter(|| {
            rust_kzg_blst_cuda::multi_scalar_mult(&affines, &scalars);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = bench_g1_lincomb
}

criterion_main!(benches);
