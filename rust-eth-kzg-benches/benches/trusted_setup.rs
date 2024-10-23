use criterion::{criterion_group, criterion_main, Criterion};
use rust_eth_kzg::{DASContext, TrustedSetup, UsePrecomp};

fn bench_load_trusted_setup(
    c: &mut Criterion
) {
    c.bench_function("load_trusted_setup", |b| {

        b.iter(|| {
            let trusted_setup = TrustedSetup::default();
        
            #[cfg(feature = "parallel")]
            let _ = DASContext::with_threads(
                &trusted_setup,
                rust_eth_kzg::ThreadCount::Multi(std::thread::available_parallelism().unwrap().into()),
                UsePrecomp::Yes { width: 8 },
            );
        
            #[cfg(not(feature = "parallel"))]
            let _ = DASContext::new(&trusted_setup, UsePrecomp::Yes { width: 8 });
        });
    });
}

criterion_group!(benches, bench_load_trusted_setup);
criterion_main!(benches);
