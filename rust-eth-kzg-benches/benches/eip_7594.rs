use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use kzg_bench::benches::eip_7594::get_partial_cells;
use kzg_bench::tests::eip_4844::generate_random_blob_bytes;
use rust_eth_kzg::constants::CELLS_PER_EXT_BLOB;
use rust_eth_kzg::{DASContext, TrustedSetup};
use rust_eth_kzg::UsePrecomp;

fn bench_eip_7594_(c: &mut Criterion) {
    const MAX_COUNT: usize = 64;

    let trusted_setup = TrustedSetup::default();

    let ctx = DASContext::with_threads(
        &trusted_setup,
        rust_eth_kzg::ThreadCount::Multi(std::thread::available_parallelism().unwrap().into()),
        UsePrecomp::Yes { width: 8 },
    );

    let mut rng = rand::thread_rng();

    let blobs = (0..MAX_COUNT)
        .map(|_| {
            generate_random_blob_bytes(&mut rng)
        })
        .collect::<Vec<_>>();
    
    let mut blob_cells = Vec::with_capacity(MAX_COUNT);
    let mut blob_cell_proofs = Vec::with_capacity(MAX_COUNT);
    let mut blob_commitments = Vec::with_capacity(MAX_COUNT);

    for blob in blobs.iter() {
        let (cells, proofs) = ctx.compute_cells_and_kzg_proofs(blob).unwrap();

        blob_cells.push(cells);
        blob_cell_proofs.push(proofs);
        blob_commitments.push(ctx.blob_to_kzg_commitment(blob).unwrap());
    }

    let blob_cells = blob_cells;
    let blob_cell_proofs = blob_cell_proofs;
    let blob_commitments = blob_commitments;

    c.bench_function("compute_cells_and_kzg_proofs", |b| {
        b.iter(|| {
            ctx.compute_cells_and_kzg_proofs(&blobs[0]).unwrap();
        });
    });

    let mut group = c.benchmark_group("recover_cells_and_kzg_proofs (% missing)");
    for i in [2, 4, 8] {
        let percent_missing = 100.0 / (i as f64);
        let (cell_indices, partial_cells) = get_partial_cells(&blob_cells[0], i);

        let partial_cells = partial_cells.iter().map(|it| it.as_ref()).collect::<Vec<_>>();
        let cell_indices = cell_indices.into_iter().map(|it| it as u64).collect::<Vec<_>>();

        group.bench_function(BenchmarkId::from_parameter(percent_missing), |b| {
            b.iter(|| {
                ctx.recover_cells_and_kzg_proofs(cell_indices.clone(), partial_cells.clone()).unwrap();
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("recover_cells_and_kzg_proofs (missing)");
    for i in 1..=5 {
        let modulo = (CELLS_PER_EXT_BLOB + i - 1) / i;
        let (cell_indices, partial_cells) = get_partial_cells(&blob_cells[0], modulo);

        let partial_cells = partial_cells.iter().map(|it| it.as_ref()).collect::<Vec<_>>();
        let cell_indices = cell_indices.into_iter().map(|it| it as u64).collect::<Vec<_>>();

        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            b.iter(|| {
                ctx.recover_cells_and_kzg_proofs(cell_indices.clone(), partial_cells.clone()).unwrap();
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
                cell_commitments.push(&blob_commitments[row_index]);
                cell_indices.push(cell_index as u64);
                cells.push(cell.as_ref());
                cell_proofs.push(&blob_cell_proofs[row_index][cell_index]);
            }
        }
        
        b.iter(|| {
            ctx.verify_cell_kzg_proof_batch(cell_commitments.clone(), cell_indices.clone(), cells.clone(), cell_proofs.clone()).unwrap();
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
                    cell_commitments.push(&blob_commitments[row_index]);
                    cell_indices.push(cell_index as u64);
                    cells.push(cell.as_ref());
                    cell_proofs.push(&blob_cell_proofs[row_index][cell_index]);
                }
            }

            b.iter(|| {
                ctx.verify_cell_kzg_proof_batch(cell_commitments.clone(), cell_indices.clone(), cells.clone(), cell_proofs.clone()).unwrap();
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
                    cell_commitments.push(&blob_commitments[row_index]);
                    cell_indices.push(cell_index as u64);
                    cells.push(cell.as_ref());
                    cell_proofs.push(&blob_cell_proofs[row_index][cell_index]);
                }
            }

            b.iter(|| {
                ctx.verify_cell_kzg_proof_batch(cell_commitments.clone(), cell_indices.clone(), cells.clone(), cell_proofs.clone()).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_eip_7594_
);
criterion_main!(benches);
