use std::env;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use blst::{blst_p1s_mult_wbits, blst_p1s_mult_wbits_precompute};
use human_bytes::human_bytes;
use kzg::{msm::{msm_impls::msm, precompute::precompute}, Fr, G1Affine, G1};
use rust_kzg_blst::types::{fp::FsFp, fr::FsFr, g1::{FsG1, FsG1Affine, FsG1ProjAddAffine}};
use rust_kzg_constantine::types::{fp::CtFp, fr::CtFr, g1::{CtG1, CtG1Affine, CtG1ProjAddAffine}};
use crate_crypto_internal_eth_kzg_bls12_381::{ff::Field, fixed_base_msm_window::FixedBaseMSMPrecompWindow, group::Group};

fn bench_fixed_base_msm(c: &mut Criterion) {
    let npow: usize = env::var("BENCH_NPOW").unwrap_or("12".to_owned()).parse().unwrap();
    let npoints = 1usize << npow;

    {
        let points = (0..npoints).map(|_| {
            FsG1Affine::into_affine(&FsG1::rand()).0
        }).collect::<Vec<_>>();
        let points = [points.as_ptr(), std::ptr::null()];
        let mut group = c.benchmark_group("blst wbits initialization");
        let precomputations = (8..=10).map(|wbits| {
            let precompute_size = unsafe { blst::blst_p1s_mult_wbits_precompute_sizeof(wbits, npoints) };
            let mut precomputation = vec![blst::blst_p1_affine::default(); precompute_size / size_of::<blst::blst_p1_affine>()];

            group.bench_function(BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}, precomp_size: {}", npow, wbits, human_bytes(precompute_size as f64))), |b| {
                b.iter(|| {
                    unsafe { blst_p1s_mult_wbits_precompute(precomputation.as_mut_ptr(), wbits, points.as_ptr(), npoints) };
                });
            });

            (wbits, precomputation)
        }).collect::<Vec<_>>();
        group.finish();

        let scalars = (0..npoints).map(|_| {
            let fr = FsFr::rand().0;
            let mut scalar = blst::blst_scalar::default();
            unsafe { blst::blst_scalar_from_fr(&mut scalar, &fr); }
            scalar.b
        }).collect::<Vec<_>>();

        let mut group = c.benchmark_group("blst wbits mult");
        precomputations.into_iter().for_each(|(wbits, precomputation)| {
            let scratch_size = unsafe { blst::blst_p1s_mult_wbits_scratch_sizeof(npoints) };
            let mut scratch = vec![blst::limb_t::default(); scratch_size / size_of::<blst::limb_t>()];
            let scalars_arg = [scalars.as_ptr() as *const u8, std::ptr::null()];
            
            group.bench_function(BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}", npow, wbits)), |b| {
                b.iter(|| {
                    let mut output = blst::blst_p1::default();
                    unsafe { blst_p1s_mult_wbits(&mut output, precomputation.as_ptr(), wbits, npoints, scalars_arg.as_ptr(), 255, scratch.as_mut_ptr()); }
                })
            });
        });
        group.finish();
    }

    {
        let mut rng = rand::thread_rng();
        let points = (0..npoints).map(|_| {
            blstrs::G1Projective::random(&mut rng).into()
        }).collect::<Vec<_>>();
        let mut group = c.benchmark_group("crate-crypto wbits initialization");
        let precomputations = (8..=10).map(|wbits| {
            group.bench_function(BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}", npow, wbits)), |b| {
                b.iter(|| {
                    FixedBaseMSMPrecompWindow::new(&points, wbits)
                });
            });

            (wbits, FixedBaseMSMPrecompWindow::new(&points, wbits))
        }).collect::<Vec<_>>();
        group.finish();

        let scalars = (0..npoints).map(|_| {
            blstrs::Scalar::random(&mut rng)
        }).collect::<Vec<_>>();

        let mut group = c.benchmark_group("crate-crypto wbits mult");
        precomputations.into_iter().for_each(|(wbits, precomputation)| {
            group.bench_function(BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}", npow, wbits)), |b| {
                b.iter(|| {
                    precomputation.msm(&scalars)
                })
            });
        });
        group.finish();
    }

    {
        let points = (0..npoints).map(|_| {
            FsG1::rand()
        }).collect::<Vec<_>>();

        let table = precompute::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(&points, &[]).ok().flatten();

        if table.is_some() {
            c.bench_function(format!("rust-kzg-blst msm initialization, points: 2^{}", npow).as_str(), |b| {
                b.iter(|| {
                    precompute::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(&points, &[]).unwrap().unwrap()
                });
            });
        }
    
        let scalars = (0..npoints).map(|_| {
            FsFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-blst msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(&points, &scalars, npoints, table.as_ref());
            })
        });
    }

    {
        let points = (0..npoints).map(|_| {
            CtG1::rand()
        }).collect::<Vec<_>>();

        let table = precompute::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(&points, &[]).ok().flatten();

        if table.is_some() {
            c.bench_function(format!("rust-kzg-constantine msm initialization, points: 2^{}", npow).as_str(), |b| {
                b.iter(|| {
                    precompute::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(&points, &[]).unwrap().unwrap()
                });
            });
        }
    
        let scalars = (0..npoints).map(|_| {
            CtFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-constantine msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                msm::<CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine, CtFr>(&points, &scalars, npoints, table.as_ref());
            })
        });
    }

    {
        use rust_kzg_arkworks3::{kzg_types::{ArkG1, ArkFr}, fft_g1::g1_linear_combination};
        let points = (0..npoints).map(|_| {
            ArkG1::rand()
        }).collect::<Vec<_>>();
        
        let scalars = (0..npoints).map(|_| {
            ArkFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-arkworks3 msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                let mut output = ArkG1::default();
                g1_linear_combination(&mut output, &points, &scalars, npoints, None);
            })
        });
    }

    {
        use rust_kzg_arkworks4::kzg_types::{ArkG1, ArkFp, ArkFr, ArkG1Affine, ArkG1ProjAddAffine};
        let points = (0..npoints).map(|_| {
            ArkG1::rand()
        }).collect::<Vec<_>>();

        let table = precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(&points, &[]).ok().flatten();

        if table.is_some() {
            c.bench_function(format!("rust-kzg-arkworks4 msm initialization, points: 2^{}", npow).as_str(), |b| {
                b.iter(|| {
                    precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(&points, &[]).unwrap().unwrap()
                });
            });
        }
    
        let scalars = (0..npoints).map(|_| {
            ArkFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-arkworks4 msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(&points, &scalars, npoints, table.as_ref());
            })
        });
    }

    {
        use rust_kzg_arkworks5::kzg_types::{ArkG1, ArkFp, ArkFr, ArkG1Affine, ArkG1ProjAddAffine};
        let points = (0..npoints).map(|_| {
            ArkG1::rand()
        }).collect::<Vec<_>>();

        let table = precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(&points, &[]).ok().flatten();

        if table.is_some() {
            c.bench_function(format!("rust-kzg-arkworks5 msm initialization, points: 2^{}", npow).as_str(), |b| {
                b.iter(|| {
                    precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(&points, &[]).unwrap().unwrap()
                });
            });
        }
    
        let scalars = (0..npoints).map(|_| {
            ArkFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-arkworks5 msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(&points, &scalars, npoints, table.as_ref());
            })
        });
    }

    {
        use rust_kzg_mcl::{kzg_proofs::g1_linear_combination, types::{g1::MclG1, fr::MclFr}};
        let points = (0..npoints).map(|_| {
            MclG1::rand()
        }).collect::<Vec<_>>();
        
        let scalars = (0..npoints).map(|_| {
            MclFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-mcl msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                let mut output = MclG1::default();
                g1_linear_combination(&mut output, &points, &scalars, npoints, None);
            })
        });
    }

    {
        use rust_kzg_zkcrypto::{kzg_types::{ZG1, ZFr}, fft_g1::g1_linear_combination};
        let points = (0..npoints).map(|_| {
            ZG1::rand()
        }).collect::<Vec<_>>();
        
        let scalars = (0..npoints).map(|_| {
            ZFr::default()
        }).collect::<Vec<_>>();

        c.bench_function(format!("rust-kzg-mcl msm mult, points: 2^{}", npow).as_str(), |b| {
            b.iter(|| {
                let mut output = ZG1::default();
                g1_linear_combination(&mut output, &points, &scalars, npoints, None);
            })
        });
    }
}

criterion_group!(benches, bench_fixed_base_msm);
criterion_main!(benches);
