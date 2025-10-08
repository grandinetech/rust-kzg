use std::env;

use blst::{blst_p1_is_equal, blst_p1s_mult_wbits, blst_p1s_mult_wbits_precompute};
use crate_crypto_internal_eth_kzg_bls12_381::{
    fixed_base_msm_window::FixedBaseMSMPrecompWindow,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use human_bytes::human_bytes;
use kzg::{
    msm::{msm_impls::msm, precompute::precompute},
    Fr, G1Affine, G1, G1Mul
};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rust_kzg_blst::types::{
    fp::FsFp,
    fr::FsFr,
    g1::{FsG1, FsG1Affine, FsG1ProjAddAffine},
};
use rust_kzg_constantine::types::{
    fp::CtFp,
    fr::CtFr,
    g1::{CtG1, CtG1Affine, CtG1ProjAddAffine},
};

fn bench_fixed_base_msm(c: &mut Criterion) {
    let npow: usize = env::var("BENCH_NPOW")
        .unwrap_or("12".to_owned())
        .parse()
        .unwrap();
    let npoints = 1usize << npow;

    let mut rng = {
        let seed = env::var("SEED").unwrap_or("rand".to_owned());

        if seed == "rand" {
            rand_chacha::ChaCha8Rng::from_rng(rand::thread_rng()).unwrap()
        } else {
            rand_chacha::ChaCha8Rng::seed_from_u64(seed.parse().unwrap())
        }
    };

    let points = (0..npoints).map(|_| {
        let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();

        let p = FsG1::generator().mul(&fr);

        p.to_bytes()
    }).collect::<Vec<_>>();

    let scalars = (0..npoints).map(|_| {
        let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();

        fr.to_bytes()
    }).collect::<Vec<_>>();

    let expected_result = {
        let mut res = FsG1::zero();

        for (p, s) in points.iter().zip(scalars.iter()) {
            let p = FsG1::from_bytes(p).unwrap();
            let s = FsFr::from_bytes(s).unwrap();

            res = res.add_or_dbl(&p.mul(&s));
        }

        res.to_bytes()
    };

    {
        let points = points
            .iter()
            .map(|p| FsG1Affine::into_affine(&FsG1::from_bytes(p).unwrap()).0)
            .collect::<Vec<_>>();
        let points = [points.as_ptr(), std::ptr::null()];
        let mut group = c.benchmark_group("blst wbits initialization");
        let precomputations = option_env!("WINDOW_SIZE")
            .map(|v| {
                v.parse()
                    .expect("WINDOW_SIZE environment variable must be valid number")
            })
            .map(|v| v..=v)
            .unwrap_or(8..=10)
            .map(|wbits| {
                let precompute_size =
                    unsafe { blst::blst_p1s_mult_wbits_precompute_sizeof(wbits, npoints) };
                let mut precomputation = vec![
                    blst::blst_p1_affine::default();
                    precompute_size / size_of::<blst::blst_p1_affine>()
                ];

                group.bench_function(
                    BenchmarkId::from_parameter(format!(
                        "points: 2^{}, wbits: {}, precomp_size: {}",
                        npow,
                        wbits,
                        human_bytes(precompute_size as f64)
                    )),
                    |b| {
                        b.iter(|| {
                            unsafe {
                                blst_p1s_mult_wbits_precompute(
                                    precomputation.as_mut_ptr(),
                                    wbits,
                                    points.as_ptr(),
                                    npoints,
                                )
                            };
                        });
                    },
                );

                (wbits, precomputation)
            })
            .collect::<Vec<_>>();
        group.finish();

        let scalars = scalars
            .iter()
            .map(|s| {
                let mut scalar = blst::blst_scalar::default();
                unsafe {
                    blst::blst_scalar_from_bendian(&mut scalar, s.as_ptr());
                }
                scalar.b
            })
            .collect::<Vec<_>>();

        let expected_result = FsG1::from_bytes(&expected_result).unwrap().0;

        let mut group = c.benchmark_group("blst wbits mult");
        precomputations
            .into_iter()
            .for_each(|(wbits, precomputation)| {
                let scratch_size = unsafe { blst::blst_p1s_mult_wbits_scratch_sizeof(npoints) };
                let mut scratch =
                    vec![blst::limb_t::default(); scratch_size / size_of::<blst::limb_t>()];
                let scalars_arg = [scalars.as_ptr() as *const u8, std::ptr::null()];

                group.bench_function(
                    BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}", npow, wbits)),
                    |b| {
                        b.iter(|| {
                            let mut output = blst::blst_p1::default();
                            unsafe {
                                blst_p1s_mult_wbits(
                                    &mut output,
                                    precomputation.as_ptr(),
                                    wbits,
                                    npoints,
                                    scalars_arg.as_ptr(),
                                    255,
                                    scratch.as_mut_ptr(),
                                );
                            }

                            assert!(unsafe { blst_p1_is_equal(&output, &expected_result) });
                        })
                    },
                );
            });
        group.finish();
    }

    {
        let points = points
            .iter()
            .map(|p| blstrs::G1Projective::from_compressed(p).unwrap().into())
            .collect::<Vec<_>>();
        let mut group = c.benchmark_group("crate-crypto wbits initialization");
        let precomputations = option_env!("WINDOW_SIZE")
            .map(|v| {
                v.parse()
                    .expect("WINDOW_SIZE environment variable must be valid number")
            })
            .map(|v| v..=v)
            .unwrap_or(8..=10)
            .map(|wbits| {
                group.bench_function(
                    BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}", npow, wbits)),
                    |b| {
                        b.iter(|| FixedBaseMSMPrecompWindow::new(&points, wbits));
                    },
                );

                (wbits, FixedBaseMSMPrecompWindow::new(&points, wbits))
            })
            .collect::<Vec<_>>();
        group.finish();

        let scalars = scalars
            .iter()
            .map(|s| blstrs::Scalar::from_bytes_be(s).unwrap())
            .collect::<Vec<_>>();

        let expected_result = blstrs::G1Projective::from_compressed(&expected_result).unwrap();

        let mut group = c.benchmark_group("crate-crypto wbits mult");
        precomputations
            .into_iter()
            .for_each(|(wbits, precomputation)| {
                group.bench_function(
                    BenchmarkId::from_parameter(format!("points: 2^{}, wbits: {}", npow, wbits)),
                    |b| b.iter(|| {
                        let result = precomputation.msm(&scalars);

                        assert_eq!(result, expected_result);
                    }),
                );
            });
        group.finish();
    }

    {
        let points = points.iter().map(|p| FsG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let table = precompute::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(&points, &[])
            .ok()
            .flatten();

        if table.is_some() {
            c.bench_function(
                format!("rust-kzg-blst msm initialization, points: 2^{}", npow).as_str(),
                |b| {
                    b.iter(|| {
                        precompute::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(&points, &[])
                            .unwrap()
                            .unwrap()
                    });
                },
            );
        }

        let scalars = scalars.iter().map(|s| FsFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        let expected_result = FsG1::from_bytes(&expected_result).unwrap();

        c.bench_function(
            format!("rust-kzg-blst msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    let result = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(
                        &points,
                        &scalars,
                        npoints,
                        table.as_ref(),
                    );

                    assert!(result.equals(&expected_result));
                })
            },
        );
    }

    {
        let points = points.iter().map(|p| CtG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let table = precompute::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(&points, &[])
            .ok()
            .flatten();

        if table.is_some() {
            c.bench_function(
                format!(
                    "rust-kzg-constantine msm initialization, points: 2^{}",
                    npow
                )
                .as_str(),
                |b| {
                    b.iter(|| {
                        precompute::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(&points, &[])
                            .unwrap()
                            .unwrap()
                    });
                },
            );
        }

        let scalars = scalars.iter().map(|s| CtFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        let expected_result = CtG1::from_bytes(&expected_result).unwrap();

        c.bench_function(
            format!("rust-kzg-constantine msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    let result = msm::<CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine, CtFr>(
                        &points,
                        &scalars,
                        npoints,
                        table.as_ref(),
                    );

                    assert!(result.equals(&expected_result));
                })
            },
        );
    }

    {
        use rust_kzg_arkworks3::{
            fft_g1::g1_linear_combination,
            kzg_types::{ArkFr, ArkG1},
        };
        let points = points.iter().map(|p| ArkG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let scalars = scalars.iter().map(|s| ArkFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        let expected_result = ArkG1::from_bytes(&expected_result).unwrap();

        c.bench_function(
            format!("rust-kzg-arkworks3 msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    let mut output = ArkG1::default();
                    g1_linear_combination(&mut output, &points, &scalars, npoints, None);

                    assert!(output.equals(&expected_result));
                })
            },
        );
    }

    {
        use rust_kzg_arkworks4::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG1ProjAddAffine};
        let points = points.iter().map(|p| ArkG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let table =
            precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(&points, &[])
                .ok()
                .flatten();

        if table.is_some() {
            c.bench_function(
                format!("rust-kzg-arkworks4 msm initialization, points: 2^{}", npow).as_str(),
                |b| {
                    b.iter(|| {
                        precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(
                            &points,
                            &[],
                        )
                        .unwrap()
                        .unwrap()
                    });
                },
            );
        }

        let scalars = scalars.iter().map(|s| ArkFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        c.bench_function(
            format!("rust-kzg-arkworks4 msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(
                        &points,
                        &scalars,
                        npoints,
                        table.as_ref(),
                    );
                })
            },
        );
    }

    {
        use rust_kzg_arkworks5::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG1ProjAddAffine};
        let points = points.iter().map(|p| ArkG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let table =
            precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(&points, &[])
                .ok()
                .flatten();

        if table.is_some() {
            c.bench_function(
                format!("rust-kzg-arkworks5 msm initialization, points: 2^{}", npow).as_str(),
                |b| {
                    b.iter(|| {
                        precompute::<ArkFr, ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine>(
                            &points,
                            &[],
                        )
                        .unwrap()
                        .unwrap()
                    });
                },
            );
        }

        let scalars = scalars.iter().map(|s| ArkFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        let expected_result = ArkG1::from_bytes(&expected_result).unwrap();

        c.bench_function(
            format!("rust-kzg-arkworks5 msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    let result = msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(
                        &points,
                        &scalars,
                        npoints,
                        table.as_ref(),
                    );
                    assert!(result.equals(&expected_result));
                })
            },
        );
    }

    {
        use rust_kzg_mcl::{
            kzg_proofs::g1_linear_combination,
            types::{fr::MclFr, g1::MclG1},
        };
        let points = points.iter().map(|p| MclG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let scalars = scalars.iter().map(|s| MclFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        let expected_result = MclG1::from_bytes(&expected_result).unwrap();

        c.bench_function(
            format!("rust-kzg-mcl msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    let mut output = MclG1::default();
                    g1_linear_combination(&mut output, &points, &scalars, npoints, None);

                    assert!(output.equals(&expected_result));
                })
            },
        );
    }

    {
        use rust_kzg_zkcrypto::{
            fft_g1::g1_linear_combination,
            kzg_types::{ZFr, ZG1},
        };
        let points = points.iter().map(|p| ZG1::from_bytes(p).unwrap()).collect::<Vec<_>>();

        let scalars = scalars.iter().map(|s| ZFr::from_bytes(s).unwrap()).collect::<Vec<_>>();

        let expected_result = ZG1::from_bytes(&expected_result).unwrap();

        c.bench_function(
            format!("rust-kzg-zkcrypto msm mult, points: 2^{}", npow).as_str(),
            |b| {
                b.iter(|| {
                    let mut output = ZG1::default();
                    g1_linear_combination(&mut output, &points, &scalars, npoints, None);

                    assert!(output.equals(&expected_result));
                })
            },
        );
    }
}

criterion_group!(benches, bench_fixed_base_msm);
criterion_main!(benches);
