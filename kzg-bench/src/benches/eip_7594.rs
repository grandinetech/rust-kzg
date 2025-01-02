use std::path::PathBuf;

use crate::tests::eip_4844::generate_random_blob_bytes;
use criterion::{BenchmarkId, Criterion};
use kzg::{eip_4844::TRUSTED_SETUP_PATH, eth, EcBackend, DAS};

pub fn get_partial_cells<T: Clone>(
    cells: &[T],
    chunk_size: usize,
    m: usize,
) -> (Vec<usize>, Vec<T>) {
    let mut cell_indices = Vec::new();
    let mut partial_cells = Vec::new();

    for (i, cell) in cells.chunks(chunk_size).enumerate() {
        if i % m != 0 {
            cell_indices.push(i);
            partial_cells.extend_from_slice(cell);
        }
    }

    (cell_indices, partial_cells)
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn bench_eip_7594<B: EcBackend>(
    c: &mut Criterion,
    load_trusted_setup: &dyn Fn(&str) -> Result<B::KZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<B::Fr>, String>,
    blob_to_kzg_commitment: &dyn Fn(&[B::Fr], &B::KZGSettings) -> Result<B::G1, String>,
) {
    let ts = load_trusted_setup(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(TRUSTED_SETUP_PATH)
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let mut rng = rand::thread_rng();

    const MAX_COUNT: usize = 64;

    let blobs: Vec<[u8; eth::BYTES_PER_BLOB]> = (0..MAX_COUNT)
        .map(|_| generate_random_blob_bytes(&mut rng))
        .collect();

    let mut blob_cells = Vec::with_capacity(MAX_COUNT);
    let mut blob_cell_proofs = Vec::with_capacity(MAX_COUNT);
    let mut blob_commitments = Vec::with_capacity(MAX_COUNT);

    for blob in blobs.iter() {
        let mut cells =
            vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];
        let mut proofs = vec![B::G1::default(); eth::CELLS_PER_EXT_BLOB];

        let blob = bytes_to_blob(blob).unwrap();
        <B::KZGSettings as DAS<B>>::compute_cells_and_kzg_proofs(
            &ts,
            Some(&mut cells),
            Some(&mut proofs),
            &blob,
        )
        .unwrap();
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
                vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];
            let mut recv_proofs = vec![B::G1::default(); eth::CELLS_PER_EXT_BLOB];

            <B::KZGSettings as DAS<B>>::compute_cells_and_kzg_proofs(
                &ts,
                Some(&mut recv_cells),
                Some(&mut recv_proofs),
                &blob,
            )
            .unwrap();
        });
    });

    let mut group = c.benchmark_group("recover_cells_and_kzg_proofs (% missing)");
    for i in [2, 4, 8] {
        let percent_missing = 100.0 / (i as f64);
        let (cell_indices, partial_cells) =
            get_partial_cells(&blob_cells[0], eth::FIELD_ELEMENTS_PER_CELL, i);

        group.bench_function(BenchmarkId::from_parameter(percent_missing), |b| {
            b.iter(|| {
                let mut recv_cells =
                    vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];

                let mut recv_proofs = vec![B::G1::default(); eth::CELLS_PER_EXT_BLOB];

                <B::KZGSettings as DAS<B>>::recover_cells_and_kzg_proofs(
                    &ts,
                    &mut recv_cells,
                    Some(&mut recv_proofs),
                    &cell_indices,
                    &partial_cells,
                )
                .unwrap();
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("recover_cells_and_kzg_proofs (missing)");
    for i in 1..=5 {
        let modulo = (eth::CELLS_PER_EXT_BLOB + i - 1) / i;
        let (cell_indices, partial_cells) =
            get_partial_cells(&blob_cells[0], eth::FIELD_ELEMENTS_PER_CELL, modulo);

        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            b.iter(|| {
                let mut recv_cells =
                    vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];

                let mut recv_proofs = vec![B::G1::default(); eth::CELLS_PER_EXT_BLOB];

                <B::KZGSettings as DAS<B>>::recover_cells_and_kzg_proofs(
                    &ts,
                    &mut recv_cells,
                    Some(&mut recv_proofs),
                    &cell_indices,
                    &partial_cells,
                )
                .unwrap();
            });
        });
    }
    group.finish();

    c.bench_function("verify_cell_kzg_proof_batch", |b| {
        let mut cell_commitments = Vec::with_capacity(MAX_COUNT * eth::CELLS_PER_EXT_BLOB);
        let mut cell_indices = Vec::with_capacity(MAX_COUNT * eth::CELLS_PER_EXT_BLOB);
        let mut cells = Vec::with_capacity(MAX_COUNT * eth::CELLS_PER_EXT_BLOB);
        let mut cell_proofs = Vec::with_capacity(MAX_COUNT * eth::CELLS_PER_EXT_BLOB);

        for (row_index, blob_cell) in blob_cells.iter().enumerate() {
            for (cell_index, cell) in blob_cell.chunks(eth::FIELD_ELEMENTS_PER_CELL).enumerate() {
                cell_commitments.push(blob_commitments[row_index].clone());
                cell_indices.push(cell_index);
                cells.extend_from_slice(cell);
                cell_proofs.push(blob_cell_proofs[row_index][cell_index].clone());
            }
        }

        b.iter(|| {
            let result = <B::KZGSettings as DAS<B>>::verify_cell_kzg_proof_batch(
                &ts,
                &cell_commitments,
                &cell_indices,
                &cells,
                &cell_proofs,
            )
            .unwrap();
            assert!(result);
        });
    });

    let mut group = c.benchmark_group("verify_cell_kzg_proof_batch (rows)");
    for i in (0..=MAX_COUNT.ilog2()).map(|exp| 2usize.pow(exp)) {
        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            let mut cell_commitments = Vec::with_capacity(i * eth::CELLS_PER_EXT_BLOB);
            let mut cell_indices = Vec::with_capacity(i * eth::CELLS_PER_EXT_BLOB);
            let mut cells = Vec::with_capacity(i * eth::CELLS_PER_EXT_BLOB);
            let mut cell_proofs = Vec::with_capacity(i * eth::CELLS_PER_EXT_BLOB);

            for (row_index, blob_cell) in blob_cells.iter().take(i).enumerate() {
                for (cell_index, cell) in blob_cell.chunks(eth::FIELD_ELEMENTS_PER_CELL).enumerate()
                {
                    cell_commitments.push(blob_commitments[row_index].clone());
                    cell_indices.push(cell_index);
                    cells.extend_from_slice(cell);
                    cell_proofs.push(blob_cell_proofs[row_index][cell_index].clone());
                }
            }

            b.iter(|| {
                let result = <B::KZGSettings as DAS<B>>::verify_cell_kzg_proof_batch(
                    &ts,
                    &cell_commitments,
                    &cell_indices,
                    &cells,
                    &cell_proofs,
                )
                .unwrap();
                assert!(result);
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("verify_cell_kzg_proof_batch (columns)");
    for i in (0..=eth::CELLS_PER_EXT_BLOB.ilog2()).map(|exp| 2usize.pow(exp)) {
        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            let mut cell_commitments = Vec::with_capacity(MAX_COUNT * i);
            let mut cell_indices = Vec::with_capacity(MAX_COUNT * i);
            let mut cells = Vec::with_capacity(MAX_COUNT * i);
            let mut cell_proofs = Vec::with_capacity(MAX_COUNT * i);

            for (row_index, blob_cell) in blob_cells.iter().enumerate() {
                for (cell_index, cell) in blob_cell
                    .chunks(eth::FIELD_ELEMENTS_PER_CELL)
                    .take(i)
                    .enumerate()
                {
                    cell_commitments.push(blob_commitments[row_index].clone());
                    cell_indices.push(cell_index);
                    cells.extend_from_slice(cell);
                    cell_proofs.push(blob_cell_proofs[row_index][cell_index].clone());
                }
            }

            b.iter(|| {
                let result = <B::KZGSettings as DAS<B>>::verify_cell_kzg_proof_batch(
                    &ts,
                    &cell_commitments,
                    &cell_indices,
                    &cells,
                    &cell_proofs,
                )
                .unwrap();
                assert!(result);
            });
        });
    }
    group.finish();
}
