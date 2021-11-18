use criterion::Criterion;
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};

pub const SECRET: [u8; 32usize] = [0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc,
    0x53, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

pub fn kzg_proof<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>)
) {
    for scale in 1..16 {
        let fs = TFFTSettings::new(scale as usize).unwrap();
        let (s1, s2) = generate_trusted_setup(fs.get_max_width(), SECRET);
        let ks = TKZGSettings::new(&s1, &s2, fs.get_max_width(), &fs).unwrap();
        let mut poly = TPoly::new(fs.get_max_width()).unwrap();
        for i in 0..fs.get_max_width() {
            poly.set_coeff_at(i, &TFr::rand());
        }
        let id = format!("bench_commit_to_poly scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| ks.commit_to_poly(&poly).unwrap()));
    }
}
