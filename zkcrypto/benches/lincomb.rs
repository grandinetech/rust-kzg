use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use zkcrypto::curve::multiscalar_mul::msm_variable_base;
use zkcrypto::kzg_types::ZkG1Projective;
use zkcrypto::zkfr::blsScalar;

fn g1_linear_combination(
    out: &mut ZkG1Projective,
    points: &[ZkG1Projective],
    scalars: &[blsScalar],
    _len: usize,
) {
    *out = msm_variable_base(points, scalars);
}

fn bench_g1_lincomb_(c: &mut Criterion) {
    bench_g1_lincomb::<blsScalar, ZkG1Projective>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
