use criterion::{criterion_group, criterion_main, Criterion};
use kzg::eip_4844::{blob_to_kzg_commitment_rust, bytes_to_blob};
use kzg_bench::benches::eip_7594::bench_eip_7594;
use rust_kzg_arkworks::{eip_4844::load_trusted_setup_filename_rust, eip_7594::ArkBackend};

fn bench_eip_7594_(c: &mut Criterion) {
    bench_eip_7594::<ArkBackend>(
        c,
        &load_trusted_setup_filename_rust,
        &bytes_to_blob,
        &blob_to_kzg_commitment_rust,
    );
}

criterion_group!(benches, bench_eip_7594_);
criterion_main!(benches);
