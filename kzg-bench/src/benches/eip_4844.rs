use std::env::set_current_dir;

use crate::tests::eip_4844::{generate_random_blob_bytes, generate_random_field_element_bytes};
use criterion::{BenchmarkId, Criterion};
use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn bench_eip_4844<
    TFr: Fr + Copy,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    c: &mut Criterion,
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> (TG1, TFr),
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> TG1,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> bool,
    verify_blob_kzg_proof_batch: &dyn Fn(&[Vec<TFr>], &[TG1], &[TG1], &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");
    let mut rng = rand::thread_rng();

    const MAX_COUNT: usize = 64;

    let blobs: Vec<Vec<TFr>> = (0..MAX_COUNT)
        .map(|_| {
            generate_random_blob_bytes(&mut rng)
                .chunks(BYTES_PER_FIELD_ELEMENT)
                .map(|chunk| {
                    bytes_to_bls_field(
                        chunk
                            .try_into()
                            .expect("Chunked into incorrect number of bytes"),
                    )
                    .unwrap()
                })
                .collect()
        })
        .collect();

    let commitments: Vec<TG1> = blobs
        .iter()
        .map(|blob| blob_to_kzg_commitment(blob, &ts))
        .collect();

    let proofs: Vec<TG1> = blobs
        .iter()
        .zip(commitments.iter())
        .map(|(blob, commitment)| compute_blob_kzg_proof(blob, commitment, &ts))
        .collect();

    let fields: Vec<TFr> = (0..MAX_COUNT)
        .map(|_| {
            let fr_bytes = generate_random_field_element_bytes(&mut rng);
            bytes_to_bls_field(&fr_bytes).unwrap()
        })
        .collect();

    c.bench_function("blob_to_kzg_commitment", |b| {
        b.iter(|| blob_to_kzg_commitment(blobs.first().unwrap(), &ts))
    });

    c.bench_function("compute_kzg_proof", |b| {
        b.iter(|| compute_kzg_proof(blobs.first().unwrap(), fields.first().unwrap(), &ts))
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
        })
    });

    let mut group = c.benchmark_group("verify_blob_kzg_proof_batch");
    for count in [1, 2, 4, 8, 16, 32, 64] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                verify_blob_kzg_proof_batch(
                    &blobs
                        .clone()
                        .into_iter()
                        .take(count)
                        .collect::<Vec<Vec<TFr>>>(),
                    &commitments
                        .clone()
                        .into_iter()
                        .take(count)
                        .collect::<Vec<TG1>>(),
                    &proofs.clone().into_iter().take(count).collect::<Vec<TG1>>(),
                    &ts,
                )
            })
        });
    }
    group.finish();
}
