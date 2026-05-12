use core::mem::size_of;
use std::env;

use blst::{blst_p1_is_equal, blst_p1s_mult_wbits, blst_p1s_mult_wbits_precompute};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use kzg::{
    msm::{msm_impls::msm, precompute::precompute},
    Fr, G1Affine, G1Mul, G1,
};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rust_kzg_blst::types::{
    fp::FsFp,
    fr::FsFr,
    g1::{FsG1, FsG1Affine, FsG1ProjAddAffine},
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

    let points = (0..npoints)
        .map(|_| {
            let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();
            FsG1::generator().mul(&fr).to_bytes()
        })
        .collect::<Vec<_>>();

    let scalars = (0..npoints)
        .map(|_| {
            let fr = FsFr::from_bytes_unchecked(&rng.gen::<[u8; 32]>()).unwrap();
            fr.to_bytes()
        })
        .collect::<Vec<_>>();

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
        let points_arg = [points.as_ptr(), std::ptr::null()];

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
                println!("blst wbits table size in bytes: {}", precompute_size);
                let mut precomputation = vec![
                    blst::blst_p1_affine::default();
                    precompute_size / size_of::<blst::blst_p1_affine>()
                ];

                unsafe {
                    blst_p1s_mult_wbits_precompute(
                        precomputation.as_mut_ptr(),
                        wbits,
                        points_arg.as_ptr(),
                        npoints,
                    )
                };

                (wbits, precomputation)
            })
            .collect::<Vec<_>>();

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
            .map(|p| FsG1::from_bytes(p).unwrap())
            .collect::<Vec<_>>();
        let table = precompute::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(&points, &[])
            .ok()
            .flatten();
        let scalars = scalars
            .iter()
            .map(|s| FsFr::from_bytes(s).unwrap())
            .collect::<Vec<_>>();
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
}

criterion_group!(benches, bench_fixed_base_msm);
criterion_main!(benches);
