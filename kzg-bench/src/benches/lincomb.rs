use criterion::Criterion;
use kzg::{Fr, G1};

#[allow(clippy::type_complexity)]
pub fn bench_g1_lincomb<TFr: Fr + Copy, TG1: G1 + Copy>(
    c: &mut Criterion,
    g1_linear_combination: &dyn Fn(&mut TG1, &[TG1], &[TFr], usize),
) {
    const NUM_POINTS: usize = 4096;

    let points = [TG1::rand(); NUM_POINTS];
    let scalars = [TFr::rand(); NUM_POINTS];

    let id = format!("bench_g1_lincomb points: '{}'", NUM_POINTS);
    c.bench_function(&id, |b| {
        b.iter(|| {
            let mut out = TG1::default();
            g1_linear_combination(&mut out, points.as_slice(), scalars.as_slice(), NUM_POINTS)
        })
    });
}
