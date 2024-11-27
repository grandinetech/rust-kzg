use criterion::Criterion;
use kzg::{EcBackend, FFTSettings, Fr, KZGSettings, Poly, Preset};

pub const SECRET: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const BENCH_SCALE: usize = 15;

struct TestPreset;

impl Preset for TestPreset {
    const FIELD_ELEMENTS_PER_BLOB: usize = 32768;
    const FIELD_ELEMENTS_PER_EXT_BLOB: usize = 65536;
    const CELLS_PER_EXT_BLOB: usize = 512;
}

#[allow(clippy::type_complexity)]
pub fn bench_commit_to_poly<B: EcBackend>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let fs = B::FFTSettings::new(BENCH_SCALE).unwrap();
    let (s1, s2, s3) = generate_trusted_setup(fs.get_max_width(), SECRET);
    let ks = B::KZGSettings::new_for_preset::<64, TestPreset>(&s1, &s2, &s3, &fs).unwrap();
    let mut poly = B::Poly::new(fs.get_max_width());
    for i in 0..fs.get_max_width() {
        poly.set_coeff_at(i, &B::Fr::rand());
    }
    let id = format!("bench_commit_to_poly scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| b.iter(|| ks.commit_to_poly(&poly).unwrap()));
}

#[allow(clippy::type_complexity)]
pub fn bench_compute_proof_single<B: EcBackend>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let fs = B::FFTSettings::new(BENCH_SCALE).unwrap();
    let (s1, s2, s3) = generate_trusted_setup(fs.get_max_width(), SECRET);
    let ks = B::KZGSettings::new_for_preset::<64, TestPreset>(&s1, &s2, &s3, &fs).unwrap();
    let mut poly = B::Poly::new(fs.get_max_width());
    for i in 0..fs.get_max_width() {
        poly.set_coeff_at(i, &B::Fr::rand());
    }
    let id = format!("bench_compute_proof_single scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| {
        b.iter(|| ks.compute_proof_single(&poly, &B::Fr::rand()).unwrap())
    });
}
