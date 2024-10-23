use std::path::PathBuf;

use crate::tests::eip_4844::generate_random_blob_bytes;
use criterion::{BenchmarkId, Criterion};
use kzg::eip_4844::{BYTES_PER_BLOB, CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_CELL, TRUSTED_SETUP_PATH};
use kzg::{FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings, Poly, G1, G2};

pub fn get_partial_cells<T: Clone>(cells: &[T], m: usize) -> (Vec<usize>, Vec<T>) {
    let mut cell_indices = Vec::new();
    let mut partial_cells = Vec::new();

    for (i, cell) in cells.iter().enumerate() {
        if i % m != 0 {
            cell_indices.push(i);
            partial_cells.push(cell.clone());
        }
    }

    (cell_indices, partial_cells)
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn bench_eip_7594<
    TFr: Fr + std::fmt::Debug,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    c: &mut Criterion,
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    compute_cells_and_kzg_proofs: &dyn Fn(
        Option<&mut [[TFr; FIELD_ELEMENTS_PER_CELL]]>,
        Option<&mut [TG1]>,
        &[TFr],
        &TKZGSettings,
    ) -> Result<(), String>,
    recover_cells_and_kzg_proofs: &dyn Fn(
        &mut [[TFr; FIELD_ELEMENTS_PER_CELL]],
        Option<&mut [TG1]>,
        &[usize],
        &[[TFr; FIELD_ELEMENTS_PER_CELL]],
        &TKZGSettings,
    ) -> Result<(), String>,
    verify_cell_kzg_proof_batch: &dyn Fn(
        &[TG1],
        &[usize],
        &[[TFr; FIELD_ELEMENTS_PER_CELL]],
        &[TG1],
        &TKZGSettings,
    ) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TRUSTED_SETUP_PATH).to_str().unwrap()
    ).unwrap();
    let mut rng = rand::thread_rng();

    const MAX_COUNT: usize = 64;

    let blobs: Vec<[u8; BYTES_PER_BLOB]> = (0..MAX_COUNT)
        .map(|_| {
            generate_random_blob_bytes(&mut rng)
        })
        .collect();

    let mut blob_cells = Vec::with_capacity(MAX_COUNT);
    let mut blob_cell_proofs = Vec::with_capacity(MAX_COUNT);
    let mut blob_commitments = Vec::with_capacity(MAX_COUNT);

    for blob in blobs.iter() {
        let mut cells = vec![core::array::from_fn::<_, FIELD_ELEMENTS_PER_CELL, _>(|_| TFr::default()); CELLS_PER_EXT_BLOB];
        let mut proofs = vec![TG1::default(); CELLS_PER_EXT_BLOB];
        
        let blob = bytes_to_blob(blob).unwrap();
        compute_cells_and_kzg_proofs(Some(&mut cells), Some(&mut proofs), &blob, &ts).unwrap();
        blob_cells.push(cells);
        blob_cell_proofs.push(proofs);
        blob_commitments.push(blob_to_kzg_commitment(&blob, &ts).unwrap());
    }

    let blob_cells = blob_cells;
    let blob_cell_proofs = blob_cell_proofs;
    let blob_commitments = blob_commitments;

    c.bench_function("compute_cells_and_kzg_proofs", |b| {
        b.iter(|| {
            let blob_bytes = blobs.first().unwrap();
            let blob = bytes_to_blob(blob_bytes).unwrap();

            let mut recv_cells =
            vec![
                core::array::from_fn::<_, FIELD_ELEMENTS_PER_CELL, _>(|_| TFr::default());
                CELLS_PER_EXT_BLOB
            ];
            let mut recv_proofs = vec![TG1::default(); CELLS_PER_EXT_BLOB];

            compute_cells_and_kzg_proofs(Some(&mut recv_cells), Some(&mut recv_proofs), &blob, &ts).unwrap();
        });
    });

    let mut group = c.benchmark_group("recover_cells_and_kzg_proofs (% missing)");
    for i in [2, 4, 8] {
        let percent_missing = 100.0 / (i as f64);
        let (cell_indices, partial_cells) = get_partial_cells(&blob_cells[0], i);

        group.bench_function(BenchmarkId::from_parameter(percent_missing), |b| {
            b.iter(|| {
                let mut recv_cells = vec![
                    vec![TFr::default(); FIELD_ELEMENTS_PER_CELL]
                        .try_into()
                        .unwrap();
                    CELLS_PER_EXT_BLOB
                ];
        
                let mut recv_proofs = vec![TG1::default(); CELLS_PER_EXT_BLOB];

                recover_cells_and_kzg_proofs(&mut recv_cells, Some(&mut recv_proofs), &cell_indices, &partial_cells, &ts).unwrap();
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("recover_cells_and_kzg_proofs (missing)");
    for i in 1..=5 {
        let modulo = (CELLS_PER_EXT_BLOB + i - 1) / i;
        let (cell_indices, partial_cells) = get_partial_cells(&blob_cells[0], modulo);

        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            b.iter(|| {
                let mut recv_cells = vec![
                    vec![TFr::default(); FIELD_ELEMENTS_PER_CELL]
                        .try_into()
                        .unwrap();
                    CELLS_PER_EXT_BLOB
                ];
        
                let mut recv_proofs = vec![TG1::default(); CELLS_PER_EXT_BLOB];

                recover_cells_and_kzg_proofs(&mut recv_cells, Some(&mut recv_proofs), &cell_indices, &partial_cells, &ts).unwrap();
            });
        });
    }
    group.finish();

    c.bench_function("verify_cell_kzg_proof_batch", |b| {
        let mut cell_commitments = Vec::with_capacity(MAX_COUNT * CELLS_PER_EXT_BLOB);
        let mut cell_indices = Vec::with_capacity(MAX_COUNT * CELLS_PER_EXT_BLOB);
        let mut cells = Vec::with_capacity(MAX_COUNT * CELLS_PER_EXT_BLOB);
        let mut cell_proofs = Vec::with_capacity(MAX_COUNT * CELLS_PER_EXT_BLOB);
        
        for (row_index, blob_cell) in blob_cells.iter().enumerate() {
            for (cell_index, cell) in blob_cell.iter().enumerate() {
                cell_commitments.push(blob_commitments[row_index].clone());
                cell_indices.push(cell_index);
                cells.push(cell.clone());
                cell_proofs.push(blob_cell_proofs[row_index][cell_index].clone());
            }
        }
        
        b.iter(|| {
            let result = verify_cell_kzg_proof_batch(&cell_commitments, &cell_indices, &cells, &cell_proofs, &ts).unwrap();
            assert!(result);
        });
    });

    let mut group = c.benchmark_group("verify_cell_kzg_proof_batch (rows)");
    for i in (0..=MAX_COUNT.ilog2()).map(|exp| 2usize.pow(exp)) {
        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            let mut cell_commitments = Vec::with_capacity(i * CELLS_PER_EXT_BLOB);
            let mut cell_indices = Vec::with_capacity(i * CELLS_PER_EXT_BLOB);
            let mut cells = Vec::with_capacity(i * CELLS_PER_EXT_BLOB);
            let mut cell_proofs = Vec::with_capacity(i * CELLS_PER_EXT_BLOB);
            
            for (row_index, blob_cell) in blob_cells.iter().take(i).enumerate() {
                for (cell_index, cell) in blob_cell.iter().enumerate() {
                    cell_commitments.push(blob_commitments[row_index].clone());
                    cell_indices.push(cell_index);
                    cells.push(cell.clone());
                    cell_proofs.push(blob_cell_proofs[row_index][cell_index].clone());
                }
            }

            b.iter(|| {
                let result = verify_cell_kzg_proof_batch(&cell_commitments, &cell_indices, &cells, &cell_proofs, &ts).unwrap();
                assert!(result);
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("verify_cell_kzg_proof_batch (columns)");
    for i in (0..=CELLS_PER_EXT_BLOB.ilog2()).map(|exp| 2usize.pow(exp)) {
        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            let mut cell_commitments = Vec::with_capacity(MAX_COUNT * i);
            let mut cell_indices = Vec::with_capacity(MAX_COUNT * i);
            let mut cells = Vec::with_capacity(MAX_COUNT * i);
            let mut cell_proofs = Vec::with_capacity(MAX_COUNT * i);
            
            for (row_index, blob_cell) in blob_cells.iter().enumerate() {
                for (cell_index, cell) in blob_cell.iter().take(i).enumerate() {
                    cell_commitments.push(blob_commitments[row_index].clone());
                    cell_indices.push(cell_index);
                    cells.push(cell.clone());
                    cell_proofs.push(blob_cell_proofs[row_index][cell_index].clone());
                }
            }

            b.iter(|| {
                let result = verify_cell_kzg_proof_batch(&cell_commitments, &cell_indices, &cells, &cell_proofs, &ts).unwrap();
                assert!(result);
            });
        });
    }
    group.finish();
}
