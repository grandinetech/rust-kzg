use std::env::set_current_dir;

use criterion::Criterion;
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};
use rand::{rngs::StdRng, Rng, SeedableRng};

const BENCH_SCALE: usize = 15;

pub fn bench_compute_aggregate_kzg_proof<
    TFr: 'static + Fr,
    TG1: 'static + G1,
    TG2: 'static + G2,
    TPoly: 'static + Poly<TFr>,
    TFFTSettings: 'static + FFTSettings<TFr>,
    TKZGSettings: 'static + KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    c: &mut Criterion,
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    compute_aggregate_kzg_proof: &dyn Fn(&[Vec<TFr>], &TKZGSettings) -> TG1,
    blob_count: usize,
) {
    const BLOB_SIZE: usize = 4096;

    let mut rng = StdRng::seed_from_u64(0);

    let blobs = (0..blob_count)
        .map(|_| {
            (0..BLOB_SIZE)
                .map(|_| TFr::from_u64_arr(&rng.gen()))
                .collect::<Vec<TFr>>()
        })
        .collect::<Vec<Vec<TFr>>>();

    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    let id = format!(
        "bench_compute_aggregate_kzg_proof_{} scale: '{}'",
        blob_count, BENCH_SCALE
    );
    c.bench_function(&id, move |b| {
        b.iter(|| compute_aggregate_kzg_proof(&blobs, &ts))
    });
}

#[allow(clippy::type_complexity)]
pub fn bench_verify_aggregate_kzg_proof<
    TFr: 'static + Fr,
    TG1: 'static + G1,
    TG2: 'static + G2,
    TPoly: 'static + Poly<TFr>,
    TFFTSettings: 'static + FFTSettings<TFr>,
    TKZGSettings: 'static + KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    c: &mut Criterion,
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    compute_aggregate_kzg_proof: &dyn Fn(&[Vec<TFr>], &TKZGSettings) -> TG1,
    verify_aggregate_kzg_proof: &dyn Fn(&[Vec<TFr>], &[TG1], &TG1, &TKZGSettings) -> bool,
    blob_count: usize,
) {
    const BLOB_SIZE: usize = 4096;

    let mut rng = StdRng::seed_from_u64(0);

    let blobs = (0..blob_count)
        .map(|_| {
            (0..BLOB_SIZE)
                .map(|_| TFr::from_u64_arr(&rng.gen()))
                .collect::<Vec<TFr>>()
        })
        .collect::<Vec<Vec<TFr>>>();

    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    let kzg_commitments = blobs
        .iter()
        .map(|blob| blob_to_kzg_commitment(blob, &ts))
        .collect::<Vec<TG1>>();

    // Compute proof for these blobs

    let proof = compute_aggregate_kzg_proof(&blobs, &ts);

    let id = format!(
        "bench_verify_aggregate_kzg_proof scale_{}: '{}'",
        blob_count, BENCH_SCALE
    );
    c.bench_function(&id, move |b| {
        b.iter(|| verify_aggregate_kzg_proof(&blobs, &kzg_commitments, &proof, &ts))
    });
}
