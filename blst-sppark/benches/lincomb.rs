use blst::{blst_fr, blst_p1_affine, blst_scalar, blst_scalar_from_fr};
use criterion::{criterion_group, criterion_main, Criterion};
use kzg::Fr;
use kzg::G1;
use rust_kzg_blst::types::fp::FsFp;
use rust_kzg_blst::types::fr::FsFr;
use rust_kzg_blst::types::g1::{FsG1, FsG1Affine};

extern crate alloc;

fn bench_g1_lincomb(c: &mut Criterion) {
    const NUM_POINTS: usize = 1usize << 21;

    let points = (0..NUM_POINTS)
        .into_iter()
        .map(|_| FsG1::rand())
        .collect::<Vec<_>>();
    let scalars = (0..NUM_POINTS)
        .into_iter()
        .map(|_| FsFr::rand())
        .collect::<Vec<_>>();

    let affines = kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(&points);
    let scalars =
        unsafe { alloc::slice::from_raw_parts(scalars.as_ptr() as *const blst_fr, scalars.len()) };

    let affines = unsafe {
        alloc::slice::from_raw_parts(affines.as_ptr() as *const blst_p1_affine, affines.len())
    };

    let id = format!("bench_g1_lincomb points: '{}'", NUM_POINTS);
    c.bench_function(&id, |b| {
        b.iter(|| {
            rust_kzg_blst_sppark::multi_scalar_mult(&affines, &scalars);
        })
    });

    let msm = rust_kzg_blst_sppark::prepare_multi_scalar_mult(&affines);
    let id = format!("bench_g1_lincomb points (prepared, mont): '{}'", NUM_POINTS);
    c.bench_function(&id, |b| {
        b.iter(|| {
            rust_kzg_blst_sppark::multi_scalar_mult_prepared(msm, &scalars);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()/*.sample_size(100)*/;
    targets = bench_g1_lincomb
}

criterion_main!(benches);
