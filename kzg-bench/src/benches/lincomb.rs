use criterion::Criterion;
use kzg::{
    msm::precompute::{precompute, PrecomputationTable},
    Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1,
};

#[allow(clippy::type_complexity)]
pub fn bench_g1_lincomb<
    TFr: Fr + Copy,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + Copy,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    c: &mut Criterion,
    g1_linear_combination: &dyn Fn(
        &mut TG1,
        &[TG1],
        &[TFr],
        usize,
        Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine>>,
    ),
) {
    const NUM_POINTS: usize = 4096;

    let points = [TG1::rand(); NUM_POINTS];
    let scalars = [TFr::rand(); NUM_POINTS];

    let id = format!("bench_g1_lincomb points: '{}'", NUM_POINTS);
    c.bench_function(&id, |b| {
        b.iter(|| {
            let mut out = TG1::default();
            g1_linear_combination(
                &mut out,
                points.as_slice(),
                scalars.as_slice(),
                NUM_POINTS,
                None,
            )
        })
    });

    let precomputation = precompute::<TFr, TG1, TG1Fp, TG1Affine>(&points).unwrap();

    if precomputation.is_some() {
        let id = format!(
            "bench_g1_lincomb with precomputation points: '{}'",
            NUM_POINTS
        );
        c.bench_function(&id, |b| {
            b.iter(|| {
                let mut out = TG1::default();
                g1_linear_combination(
                    &mut out,
                    points.as_slice(),
                    scalars.as_slice(),
                    NUM_POINTS,
                    precomputation.as_ref(),
                )
            })
        });
    }
}
