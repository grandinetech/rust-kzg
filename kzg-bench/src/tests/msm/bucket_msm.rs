use kzg::msm::bucket_msm::BucketMSM;
use kzg::msm::glv::endomorphism;
use kzg::{G1Affine, G1Fp, G1ProjAddAffine, G1};

pub fn test_process_point_and_slices_deal_two_points<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>() {
    let window_bits = 15u32;
    let mut bucket_msm = BucketMSM::<TG1, TG1Fp, TG1Affine, TProjAddAffine>::new(
        30u32,
        window_bits,
        128u32,
        4096u32,
    );
    let p = TG1Affine::into_affine(&G1::rand());
    let q = TG1Affine::into_affine(&G1::rand());

    bucket_msm.process_point_and_slices(&p, &[1u32, 3u32]);
    bucket_msm.process_point_and_slices(&q, &[2u32, 3u32]);
    bucket_msm.process_complete();
    assert_eq!(bucket_msm.buckets[0], p);
    assert_eq!(bucket_msm.buckets[1], q);
    // FIXME: Maybe remove that to proj
    assert_eq!(
        bucket_msm.buckets[2 + (1 << bucket_msm.bucket_bits)].to_proj(),
        p.to_proj().add(&q.to_proj())
    );
}

pub fn test_process_point_and_slices_deal_three_points<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>() {
    let window_bits = 15u32;
    let mut bucket_msm = BucketMSM::<TG1, TG1Fp, TG1Affine, TProjAddAffine>::new(
        45u32,
        window_bits,
        128u32,
        4096u32,
    );
    let p = TG1Affine::into_affine(&G1::rand());
    let q = TG1Affine::into_affine(&G1::rand());
    let r = TG1Affine::into_affine(&G1::rand());

    bucket_msm.process_point_and_slices(&p, &[1u32, 3u32, 4u32]);
    bucket_msm.process_point_and_slices(&q, &[2u32, 3u32, 4u32]);
    bucket_msm.process_point_and_slices(&r, &[2u32, 3u32, 5u32]);
    bucket_msm.process_complete();
    assert_eq!(bucket_msm.buckets[0], p);
    assert_eq!(
        bucket_msm.buckets[1].to_proj(),
        q.to_proj().add(&r.to_proj())
    );
    assert_eq!(
        bucket_msm.buckets[2 + (1 << bucket_msm.bucket_bits)].to_proj(),
        p.to_proj().add(&q.to_proj()).add(&r.to_proj())
    );
    assert_eq!(
        bucket_msm.buckets[3 + (2 << bucket_msm.bucket_bits)].to_proj(),
        p.to_proj().add(&q.to_proj())
    );
    assert_eq!(bucket_msm.buckets[4 + (2 << bucket_msm.bucket_bits)], r);
}

pub fn test_process_point_and_slices_glv_deal_two_points<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>() {
    let window_bits = 15u32;
    let mut bucket_msm = BucketMSM::<TG1, TG1Fp, TG1Affine, TProjAddAffine>::new(
        30u32,
        window_bits,
        128u32,
        4096u32,
    );
    let mut p = TG1Affine::into_affine(&G1::rand());
    let mut q = TG1Affine::into_affine(&G1::rand());

    bucket_msm.process_point_and_slices_glv(&p, &[1u32, 3u32], &[4u32, 6u32], false, false);
    bucket_msm.process_point_and_slices_glv(&q, &[2u32, 3u32], &[5u32, 6u32], false, false);
    bucket_msm.process_complete();
    assert_eq!(bucket_msm.buckets[0], p);
    assert_eq!(bucket_msm.buckets[1], q);
    assert_eq!(
        bucket_msm.buckets[2 + (1 << bucket_msm.bucket_bits)].to_proj(),
        p.to_proj().add(&q.to_proj())
    );

    endomorphism(&mut p);
    endomorphism(&mut q);
    assert_eq!(bucket_msm.buckets[3], p);
    assert_eq!(bucket_msm.buckets[4], q);
    assert_eq!(
        bucket_msm.buckets[5 + (1 << bucket_msm.bucket_bits)].to_proj(),
        p.to_proj().add(&q.to_proj())
    );
}
