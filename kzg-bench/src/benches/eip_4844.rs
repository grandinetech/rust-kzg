use std::env::set_current_dir;

use crate::tests::eip_4844::{generate_random_blob_bytes, generate_random_field_element_bytes};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use kzg::eip_4844::TRUSTED_SETUP_PATH;
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn bench_eip_4844<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    c: &mut Criterion,
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> Result<(TG1, TFr), String>,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> Result<bool, String>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> Result<TG1, String>,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> Result<bool, String>,
    verify_blob_kzg_proof_batch: &dyn Fn(
        &[Vec<TFr>],
        &[TG1],
        &[TG1],
        &TKZGSettings,
    ) -> Result<bool, String>,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH).unwrap();
    let mut rng = rand::thread_rng();

    const MAX_COUNT: usize = 64;

    let blobs: Vec<Vec<TFr>> = (0..MAX_COUNT)
        .map(|_| {
            let blob_bytes = generate_random_blob_bytes(&mut rng);
            bytes_to_blob(&blob_bytes).unwrap()
        })
        .collect();

    let commitments: Vec<TG1> = blobs
        .iter()
        .map(|blob| blob_to_kzg_commitment(blob, &ts).unwrap())
        .collect();

    let proofs: Vec<TG1> = blobs
        .iter()
        .zip(commitments.iter())
        .map(|(blob, commitment)| compute_blob_kzg_proof(blob, commitment, &ts).unwrap())
        .collect();

    let fields: Vec<TFr> = (0..MAX_COUNT)
        .map(|_| {
            let fr_bytes = generate_random_field_element_bytes(&mut rng);
            TFr::from_bytes(&fr_bytes).unwrap()
        })
        .collect();

    c.bench_function("blob_to_kzg_commitment", |b| {
        b.iter(|| blob_to_kzg_commitment(blobs.first().unwrap(), &ts))
    });

    c.bench_function("compute_kzg_proof", |b| {
        b.iter(|| compute_kzg_proof(blobs.first().unwrap(), fields.first().unwrap(), &ts))
    });

    c.bench_function("verify_kzg_proof", |b| {
        b.iter(|| {
            verify_kzg_proof(
                commitments.first().unwrap(),
                fields.first().unwrap(),
                fields.first().unwrap(),
                proofs.first().unwrap(),
                &ts,
            )
            .unwrap()
        })
    });

    c.bench_function("compute_blob_kzg_proof", |b| {
        b.iter(|| compute_blob_kzg_proof(blobs.first().unwrap(), commitments.first().unwrap(), &ts))
    });

    c.bench_function("verify_blob_kzg_proof", |b| {
        b.iter(|| {
            verify_blob_kzg_proof(
                blobs.first().unwrap(),
                commitments.first().unwrap(),
                proofs.first().unwrap(),
                &ts,
            )
            .unwrap()
        })
    });

    let mut group = c.benchmark_group("verify_blob_kzg_proof_batch");
    for count in [1, 2, 4, 8, 16, 32, 64] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched_ref(
                || {
                    let blobs_subset = blobs
                        .clone()
                        .into_iter()
                        .take(count)
                        .collect::<Vec<Vec<TFr>>>();
                    let commitments_subset = commitments
                        .clone()
                        .into_iter()
                        .take(count)
                        .collect::<Vec<TG1>>();
                    let proofs_subset =
                        proofs.clone().into_iter().take(count).collect::<Vec<TG1>>();

                    (blobs_subset, commitments_subset, proofs_subset)
                },
                |(blobs_subset, commitments_subset, proofs_subset)| {
                    verify_blob_kzg_proof_batch(
                        blobs_subset,
                        commitments_subset,
                        proofs_subset,
                        &ts,
                    )
                    .unwrap()
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}
