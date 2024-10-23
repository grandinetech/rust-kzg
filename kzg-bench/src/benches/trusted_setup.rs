use criterion::Criterion;
use kzg::{
    eip_4844::TRUSTED_SETUP_PATH, FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings,
    Poly, G1, G2,
};
use std::{fs::File, io::Read, path::PathBuf};

pub fn bench_load_trusted_setup<
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
    load_trusted_setup_file: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    load_trusted_setup: &dyn Fn(&[u8], &[u8], &[u8]) -> Result<TKZGSettings, String>,
) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(TRUSTED_SETUP_PATH)
        .to_string_lossy()
        .to_string();

    c.bench_function("load_trusted_setup_file", |b| {
        b.iter(|| {
            let _ = load_trusted_setup_file(path.as_str()).unwrap();
        });
    });

    c.bench_function("load_trusted_setup", |b| {
        let mut file = File::open(path.clone()).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        let bytes = kzg::eip_4844::load_trusted_setup_string(source.as_str()).unwrap();

        b.iter(|| {
            let _ = load_trusted_setup(&bytes.0, &bytes.1, &bytes.2).unwrap();
        });
    });
}
