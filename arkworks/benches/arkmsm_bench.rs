use ark_bls12_381::G1Projective;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_kzg_arkworks::arkmsm::{utils::generate_msm_inputs, msm::VariableBaseMSM as ArkmsmMSM};
use ark_ec::VariableBaseMSM;

fn arkmsm_bench_(c: &mut Criterion) {
    for size in 8..13 {
        let (point_vec, scalar_vec) = generate_msm_inputs(1 << size);
        let point_vec = black_box(point_vec);
        let scalar_vec = black_box(scalar_vec);
        c.bench_with_input(BenchmarkId::new("Baseline", size), &size, |b, _size| {
            b.iter(|| {
                let _: G1Projective = VariableBaseMSM::msm_bigint(&point_vec, &scalar_vec);
            })
        });
        c.bench_with_input(BenchmarkId::new("ArkMSM", size), &size, |b, _size| {
            b.iter(|| {
                let _ = ArkmsmMSM::multi_scalar_mul(&point_vec, &scalar_vec);
            })
        });
    }
}

criterion_group!(benches, arkmsm_bench_);
criterion_main!(benches);