#[cfg(test)]
mod tests {
    use kzg::{
        msm::{msm_impls::msm, precompute::precompute},
        Fr, G1Mul, G1,
    };
    use rust_kzg_blst::types::{
        fp::FsFp,
        fr::FsFr,
        g1::{FsG1, FsG1Affine, FsG1ProjAddAffine},
    };

    #[test]
    fn failing_fuzz_case() {
        let points = vec![
            FsG1::zero(),
            FsG1::from_hex("0x8ef9f19afc34cd7bdd58a2850a853e0e23d962494e67c00aa8a55ba2dbd52331fc44c31e97681131c70f89f39475ad59").unwrap(),
            FsG1::zero(),
            FsG1::zero(),
            FsG1::zero(),
            FsG1::from_hex("0xaa5a3321135eb6cdea1d3c15ddb028ca4dc3d4f91c7f81e9c53b0d8429c7ef762a7afab5aca2f162f800f60f8c26d3fb").unwrap(),
            FsG1::from_hex("0xb66d5afa5472896e6759b86a68acb9ec09017084699309b108166d7360207a06790c71adc364c9c92c204f1c59e16e70").unwrap(),
            FsG1::zero(),
            FsG1::zero(),
            FsG1::zero(),
            FsG1::from_hex("0x80cd42850d03783dbab173808c98201862a9433b1b0f1e9e6af2eb1c64efa7a7566504aa13245315ef0a02e711c40e64").unwrap(),
            FsG1::zero(),
            FsG1::zero(),
            FsG1::zero(),
            FsG1::from_hex("0xa45d4a144e3df6a8a4460dd49aaa16661426ead35b9840a8793548d6dceb29a054b90f9b559e3d210cc0ceac706f2dbe").unwrap(),
            FsG1::zero(),
        ];

        let scalars = vec![
            FsFr::from_hex("0x182446eeacc5049e998c4fefecbc4ff55884b7fa0002ff02ab00c9cbc9c931c7")
                .unwrap(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::from_hex("0x000000000000000000000000000000000000000000000000000000000000006b")
                .unwrap(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
            FsFr::zero(),
        ];

        let mut expected = FsG1::zero();
        for (p, s) in points.iter().zip(scalars.iter()) {
            expected.add_or_dbl_assign(&p.mul(s));
        }

        let table = precompute(&points, &[]).unwrap();
        let received = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(
            &points,
            &scalars,
            points.len(),
            table.as_ref(),
        );

        assert!(expected.equals(&received));
    }

    #[test]
    fn test_big_diff() {
        let points = vec![
            FsG1::from_hex("0x8ef9f19afc34cd7bdd58a2850a853e0e23d962494e67c00aa8a55ba2dbd52331fc44c31e97681131c70f89f39475ad59").unwrap(),
            FsG1::from_hex("0xa45d4a144e3df6a8a4460dd49aaa16661426ead35b9840a8793548d6dceb29a054b90f9b559e3d210cc0ceac706f2dbe").unwrap(),
        ];

        let scalars = vec![
            FsFr::from_hex("0x182446eeacc5049e998c4fefecbc4ff55884b7fa0002ff02ab00c9cbc9c931c7")
                .unwrap(),
            FsFr::from_hex("0x000000000000000000000000000000000000000000000000000000000000006b")
                .unwrap(),
        ];

        let mut expected = FsG1::zero();
        for (p, s) in points.iter().zip(scalars.iter()) {
            expected.add_or_dbl_assign(&p.mul(s));
        }

        let table = precompute(&points, &[]).unwrap();
        let received = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(
            &points,
            &scalars,
            points.len(),
            table.as_ref(),
        );

        println!("Received: {:?}", received.to_bytes());
        assert!(expected.equals(&received));
    }
}
