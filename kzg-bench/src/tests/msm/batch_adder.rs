use kzg::{msm::arkmsm::batch_adder::BatchAdder, G1Affine, G1Fp, G1};

pub fn test_phase_one_zero_or_neg<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    batch_adder.batch_add_phase_one(&TG1Affine::zero(), &TG1Affine::zero(), 0);

    let p_rand = TG1::rand();
    let p_affine = TG1Affine::into_affine(&p_rand);
    let mut neg_p_affine = p_affine;
    neg_p_affine.y_mut().neg_assign();

    batch_adder.batch_add_phase_one(&p_affine, &neg_p_affine, 0);
}

pub fn test_phase_one_p_add_p<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let p = TG1Affine::into_affine(&TG1::rand());
    let acc = p;

    batch_adder.batch_add_phase_one(&acc, &p, 0);
    assert!(batch_adder.inverses[0].is_one());
    assert_eq!(batch_adder.inverse_state, p.y().add_fp(p.y()));
}

pub fn test_phase_one_p_add_q<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let p = TG1Affine::into_affine(&TG1::rand());
    let q = TG1Affine::into_affine(&TG1::rand());

    batch_adder.batch_add_phase_one(&p, &q, 0);
    assert!(batch_adder.inverses[0].is_one());
    assert_eq!(batch_adder.inverse_state, q.x().sub_fp(p.x()));
}

pub fn test_phase_one_p_add_q_twice<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let p = TG1Affine::into_affine(&TG1::rand());
    let q = TG1Affine::into_affine(&TG1::rand());

    batch_adder.batch_add_phase_one(&p, &q, 0);
    batch_adder.batch_add_phase_one(&p, &q, 0);
    assert_eq!(batch_adder.inverses[0], q.x().sub_fp(p.x()));
    assert_eq!(
        batch_adder.inverse_state,
        (q.x().sub_fp(p.x())).mul_fp(&q.x().sub_fp(p.x()))
    );
}

pub fn test_phase_two_zero_add_p<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let p = TG1Affine::into_affine(&TG1::rand());
    let mut acc = G1Affine::zero();
    batch_adder.batch_add_phase_two(&mut acc, &p, 0);
    assert_eq!(acc, p);
}

pub fn test_phase_two_p_add_neg<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let mut p = TG1Affine::into_affine(&TG1::rand());
    let mut acc = p;
    p.y_mut().neg_assign();

    batch_adder.batch_add_phase_two(&mut acc, &p, 0);
    assert_eq!(acc, G1Affine::zero());
}

pub fn test_phase_two_p_add_q<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let acc_proj = TG1::rand();
    let mut p = TG1Affine::into_affine(&acc_proj);
    let mut acc = p;
    *p.x_mut() = p.x().add_fp(p.x());

    batch_adder.inverses[0] = (p.x().sub_fp(acc.x())).inverse().unwrap();
    batch_adder.batch_add_phase_two(&mut acc, &p, 0);
    assert_eq!(acc, TG1Affine::into_affine(&acc_proj.add(&p.to_proj())));
}

pub fn test_phase_two_p_add_p<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(4);
    let acc_proj = TG1::rand();
    // let acc_proj = TG1::from_bytes(&[180, 100, 110, 26, 178, 124, 0, 160, 32, 73, 34, 58, 143, 58, 42, 253, 109, 115, 30, 187, 250, 105, 87, 92, 20, 52, 138, 74, 220, 53, 87, 230, 205, 140, 221, 30, 177, 65, 96, 179, 92, 116, 71, 234, 74, 149, 140, 221]).unwrap();
    // eprintln!("{:?}", acc_proj.to_bytes());
    let p = TG1Affine::into_affine(&acc_proj);
    let mut acc = p;

    let p_sqr = p.y().add_fp(p.y());
    let p_sqr_inv = p_sqr.inverse().unwrap();

    batch_adder.inverses[0] = p_sqr_inv;
    batch_adder.batch_add_phase_two(&mut acc, &p, 0);
    assert_eq!(acc.to_proj(), acc_proj.add_or_dbl(&p.to_proj()));
}

pub fn test_batch_add<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(10);
    let mut buckets: Vec<TG1Affine> = (0..10)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();
    let points: Vec<TG1Affine> = (0..10)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();

    let tmp = buckets.clone();
    batch_adder.batch_add(&mut buckets, &points);

    for i in 0..10 {
        assert_eq!(
            buckets[i],
            TG1Affine::into_affine(&tmp[i].to_proj().add(&points[i].to_proj()))
        );
    }
}

pub fn test_batch_add_step_n<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(10);
    let mut buckets: Vec<TG1Affine> = (0..10)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();
    let points: Vec<TG1Affine> = (0..10)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();

    let tmp = buckets.clone();
    batch_adder.batch_add_step_n(&mut buckets, 1, &points, 2, 3);

    for i in 0..3 {
        assert_eq!(
            buckets[i],
            TG1Affine::into_affine(&tmp[i].to_proj().add(&points[i * 2].to_proj()))
        );
    }
}

pub fn test_batch_add_indexed<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(10);
    let mut buckets: Vec<TG1Affine> = (0..10)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();
    let points: Vec<TG1Affine> = (0..10)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();

    let tmp = buckets.clone();
    batch_adder.batch_add_indexed(&mut buckets, &[0, 2, 4], &points, &[0, 2, 4]);

    for i in (0..5).step_by(2) {
        assert_eq!(
            buckets[i],
            TG1Affine::into_affine(&tmp[i].to_proj().add(&points[i].to_proj()))
        );
    }
}

pub fn test_batch_add_indexed_single_bucket<TG1: G1, TGFp: G1Fp, TG1Affine: G1Affine<TG1, TGFp>>() {
    let mut batch_adder = BatchAdder::<TG1, TGFp, TG1Affine>::new(1);
    let mut buckets: Vec<TG1Affine> = (0..1)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();
    let points: Vec<TG1Affine> = (0..1)
        .map(|_| TG1Affine::into_affine(&TG1::rand()))
        .collect();

    let tmp = buckets.clone();
    batch_adder.batch_add_indexed(&mut buckets, &[0], &points, &[0]);

    assert_eq!(
        buckets[0],
        TG1Affine::into_affine(&tmp[0].to_proj().add(&points[0].to_proj()))
    );
}
