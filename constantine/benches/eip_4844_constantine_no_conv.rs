// Same as eip_4844.rs, but using constantine implementations of verification functions

use std::env::set_current_dir;
use std::path::Path;

use kzg_bench::set_trusted_setup_dir;
use kzg_bench::tests::eip_4844::{generate_random_blob_bytes, generate_random_field_element_bytes};
use constantine_ethereum_kzg::EthKzgContext;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use kzg::eip_4844::{TRUSTED_SETUP_PATH, BYTES_PER_BLOB};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};
use criterion::{criterion_group, criterion_main};
use kzg::eip_4844::{
    blob_to_kzg_commitment_rust, bytes_to_blob, compute_blob_kzg_proof_rust,
    compute_kzg_proof_rust, verify_blob_kzg_proof_batch_rust, verify_blob_kzg_proof_rust,
    verify_kzg_proof_rust,
};
use kzg_bench::benches::eip_4844::bench_eip_4844;
use rust_kzg_constantine::{
    eip_4844::load_trusted_setup_filename_rust,
    mixed_kzg_settings::mixed_eip_4844::load_trusted_setup_filename_mixed,
    mixed_kzg_settings::{
        mixed_eip_4844::{
            blob_to_kzg_commitment_mixed, compute_blob_kzg_proof_mixed, compute_kzg_proof_mixed,
            verify_blob_kzg_proof_batch_mixed, verify_blob_kzg_proof_mixed, verify_kzg_proof_mixed,
        },
        mixed_kzg_settings::{MixedKzgSettings, CttContext},
    },
    types::{
        fft_settings::CtFFTSettings, fr::CtFr, g1::CtG1, g2::CtG2, kzg_settings::CtKZGSettings,
        poly::CtPoly,
    },
};

fn bench_eip_4844_constantine_no_conv_(c: &mut Criterion) {
    set_trusted_setup_dir();
    let ctx = CttContext::new(Path::new(TRUSTED_SETUP_PATH)).unwrap();
    let mut rng = rand::thread_rng();
    const MAX_COUNT: usize = 64;

    let blobs: Vec<[u8; BYTES_PER_BLOB]> = (0..MAX_COUNT)
        .map(|_| {
            generate_random_blob_bytes(&mut rng)
        })
        .collect();

    let commitments: Vec<[u8; 48]> = blobs
        .iter()
        .map(|blob| ctx.ctx.blob_to_kzg_commitment(blob).unwrap())
        .collect();

    let proofs: Vec<[u8; 48]> = blobs
        .iter()
        .zip(commitments.iter())
        .map(|(blob, commitment)| ctx.ctx.compute_blob_kzg_proof(blob, commitment).unwrap())
        .collect();

    let fields: Vec<[u8; 32]> = (0..MAX_COUNT)
        .map(|_| {
            generate_random_field_element_bytes(&mut rng)
        })
        .collect();

    c.bench_function("blob_to_kzg_commitment", |b| {
        #[cfg(feature = "parallel")]
        b.iter(|| ctx.ctx.blob_to_kzg_commitment_parallel(&ctx.pool, blobs.first().unwrap()));

        #[cfg(not(feature = "parallel"))]
        b.iter(|| ctx.ctx.blob_to_kzg_commitment(blobs.first().unwrap()));
    });

    c.bench_function("compute_kzg_proof", |b| {
        #[cfg(feature = "parallel")]
        b.iter(|| ctx.ctx.compute_kzg_proof_parallel(&ctx.pool, blobs.first().unwrap(), fields.first().unwrap()));

        #[cfg(not(feature = "parallel"))]
        b.iter(|| ctx.ctx.compute_kzg_proof(blobs.first().unwrap(), fields.first().unwrap()));
    });

    c.bench_function("verify_kzg_proof", |b| {
        b.iter(|| {
            ctx.ctx.verify_kzg_proof(
                commitments.first().unwrap(),
                fields.first().unwrap(),
                fields.first().unwrap(),
                proofs.first().unwrap(),
            )
            .unwrap()
        })
    });

    c.bench_function("compute_blob_kzg_proof", |b| {
        #[cfg(feature = "parallel")]
        b.iter(|| ctx.ctx.compute_blob_kzg_proof_parallel(&ctx.pool, blobs.first().unwrap(), commitments.first().unwrap()));

        #[cfg(not(feature = "parallel"))]
        b.iter(|| ctx.ctx.compute_blob_kzg_proof(blobs.first().unwrap(), commitments.first().unwrap()));
    });

    c.bench_function("verify_blob_kzg_proof", |b| {
        #[cfg(feature = "parallel")]
        b.iter(|| {
            ctx.ctx.verify_blob_kzg_proof_parallel(
                &ctx.pool,
                blobs.first().unwrap(),
                commitments.first().unwrap(),
                proofs.first().unwrap(),
            )
            .unwrap()
        });

        #[cfg(not(feature = "parallel"))]
        b.iter(|| {
            ctx.ctx.verify_blob_kzg_proof(
                blobs.first().unwrap(),
                commitments.first().unwrap(),
                proofs.first().unwrap(),
            )
            .unwrap()
        });
    });

    let mut group = c.benchmark_group("verify_blob_kzg_proof_batch");
    let rand_thing = [0u8; 32];
    for count in [1, 2, 4, 8, 16, 32, 64] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched_ref(
                || {
                    let blobs_subset = blobs
                        .clone()
                        .into_iter()
                        .take(count)
                        .collect::<Vec<_>>();
                    let commitments_subset = commitments
                        .clone()
                        .into_iter()
                        .take(count)
                        .collect::<Vec<_>>();
                    let proofs_subset =
                        proofs.clone().into_iter().take(count).collect::<Vec<_>>();

                    (blobs_subset, commitments_subset, proofs_subset)
                },
                |(blobs_subset, commitments_subset, proofs_subset)| {
                    #[cfg(feature = "parallel")]
                    ctx.ctx.verify_blob_kzg_proof_batch_parallel(
                        &ctx.pool,
                        blobs_subset,
                        commitments_subset,
                        proofs_subset,
                        &rand_thing,
                    )
                    .unwrap();

                    #[cfg(not(feature = "parallel"))]
                    ctx.ctx.verify_blob_kzg_proof_batch(
                        blobs_subset,
                        commitments_subset,
                        proofs_subset,
                        &rand_thing,
                    )
                    .unwrap();
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_eip_4844_constantine_no_conv_);
criterion_main!(benches);
