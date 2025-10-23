use std::{
    env,
    fs::File,
    io::{self, Write},
    iter,
    path::PathBuf,
    sync::OnceLock,
};

use kzg::{
    msm::{
        msm_impls::{msm, pippenger},
        precompute::{precompute, PrecomputationTable},
    },
    Fr, G1,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rust_kzg_blst::types::{
    fp::FsFp,
    fr::FsFr,
    g1::{FsG1, FsG1Affine, FsG1ProjAddAffine},
};
use sha2::Digest;

static NPOINTS: OnceLock<usize> = OnceLock::new();
pub fn get_npoints() -> usize {
    NPOINTS
        .get_or_init(|| {
            let npow: usize = env::var("NPOW").unwrap_or("12".to_owned()).parse().unwrap();
            let npoints: usize = 1usize << npow;
            npoints
        })
        .clone()
}

static SEED: OnceLock<<ChaCha20Rng as SeedableRng>::Seed> = OnceLock::new();
pub fn seed() -> <ChaCha20Rng as SeedableRng>::Seed {
    SEED.get_or_init(|| rand::thread_rng().gen()).clone()
}

static BACKENDS: OnceLock<Vec<Backend>> = OnceLock::new();
pub fn cached_backends(points: &[[u8; 48]]) -> &[Backend] {
    BACKENDS.get_or_init(|| {
        vec![
            Backend::blst(points),
            #[cfg(feature = "arkworks3")]
            Backend::arkworks3(points),
            #[cfg(feature = "arkworks4")]
            Backend::arkworks4(points),
            #[cfg(feature = "arkworks5")]
            Backend::arkworks5(points),
            #[cfg(feature = "constantine")]
            Backend::constantine(points),
            #[cfg(feature = "mcl")]
            Backend::mcl(points),
            #[cfg(feature = "zkcrypto")]
            Backend::zkcrypto(points),
        ]
    })
}

pub fn backends(points: &[[u8; 48]]) -> Vec<Backend> {
    vec![
        Backend::blst(points),
        #[cfg(feature = "arkworks3")]
        Backend::arkworks3(points),
        #[cfg(feature = "arkworks4")]
        Backend::arkworks4(points),
        #[cfg(feature = "arkworks5")]
        Backend::arkworks5(points),
        #[cfg(feature = "constantine")]
        Backend::constantine(points),
        #[cfg(feature = "mcl")]
        Backend::mcl(points),
        #[cfg(feature = "zkcrypto")]
        Backend::zkcrypto(points),
    ]
}

pub fn compute_expected_value(points: &[[u8; 48]], scalars: &[[u8; 32]]) -> [u8; 48] {
    let points = points
        .iter()
        .map(|bytes| FsG1::from_bytes(bytes))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let scalars = scalars
        .iter()
        .map(|bytes| FsFr::from_bytes(bytes))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    pippenger::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(&points, &scalars).to_bytes()
}

pub enum Backend {
    Blst {
        points: Vec<FsG1>,
        table: Option<PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>>,
    },
    #[cfg(feature = "arkworks3")]
    Arkworks3 {
        points: Vec<rust_kzg_arkworks3::kzg_types::ArkG1>,
        table: Option<
            PrecomputationTable<
                rust_kzg_arkworks3::kzg_types::ArkFr,
                rust_kzg_arkworks3::kzg_types::ArkG1,
                rust_kzg_arkworks3::kzg_types::ArkFp,
                rust_kzg_arkworks3::kzg_types::ArkG1Affine,
                rust_kzg_arkworks3::kzg_types::ArkG1ProjAddAffine,
            >,
        >,
    },
    #[cfg(feature = "arkworks4")]
    Arkworks4 {
        points: Vec<rust_kzg_arkworks4::kzg_types::ArkG1>,
        table: Option<
            PrecomputationTable<
                rust_kzg_arkworks4::kzg_types::ArkFr,
                rust_kzg_arkworks4::kzg_types::ArkG1,
                rust_kzg_arkworks4::kzg_types::ArkFp,
                rust_kzg_arkworks4::kzg_types::ArkG1Affine,
                rust_kzg_arkworks4::kzg_types::ArkG1ProjAddAffine,
            >,
        >,
    },
    #[cfg(feature = "arkworks5")]
    Arkworks5 {
        points: Vec<rust_kzg_arkworks5::kzg_types::ArkG1>,
        table: Option<
            PrecomputationTable<
                rust_kzg_arkworks5::kzg_types::ArkFr,
                rust_kzg_arkworks5::kzg_types::ArkG1,
                rust_kzg_arkworks5::kzg_types::ArkFp,
                rust_kzg_arkworks5::kzg_types::ArkG1Affine,
                rust_kzg_arkworks5::kzg_types::ArkG1ProjAddAffine,
            >,
        >,
    },
    #[cfg(feature = "constantine")]
    Constantine {
        points: Vec<rust_kzg_constantine::types::g1::CtG1>,
        table: Option<
            PrecomputationTable<
                rust_kzg_constantine::types::fr::CtFr,
                rust_kzg_constantine::types::g1::CtG1,
                rust_kzg_constantine::types::fp::CtFp,
                rust_kzg_constantine::types::g1::CtG1Affine,
                rust_kzg_constantine::types::g1::CtG1ProjAddAffine,
            >,
        >,
    },
    #[cfg(feature = "mcl")]
    Mcl {
        points: Vec<rust_kzg_mcl::types::g1::MclG1>,
        table: Option<
            PrecomputationTable<
                rust_kzg_mcl::types::fr::MclFr,
                rust_kzg_mcl::types::g1::MclG1,
                rust_kzg_mcl::types::fp::MclFp,
                rust_kzg_mcl::types::g1::MclG1Affine,
                rust_kzg_mcl::types::g1::MclG1ProjAddAffine,
            >,
        >,
    },
    #[cfg(feature = "zkcrypto")]
    Zkcrypto {
        points: Vec<rust_kzg_zkcrypto::kzg_types::ZG1>,
        table: Option<
            PrecomputationTable<
                rust_kzg_zkcrypto::kzg_types::ZFr,
                rust_kzg_zkcrypto::kzg_types::ZG1,
                rust_kzg_zkcrypto::kzg_types::ZFp,
                rust_kzg_zkcrypto::kzg_types::ZG1Affine,
                rust_kzg_zkcrypto::kzg_types::ZG1ProjAddAffine,
            >,
        >,
    },
}

impl Backend {
    pub fn blst(points: &[[u8; 48]]) -> Self {
        let points = points
            .iter()
            .map(|p| FsG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Blst {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    #[cfg(feature = "arkworks3")]
    pub fn arkworks3(points: &[[u8; 48]]) -> Self {
        use rust_kzg_arkworks3::kzg_types::ArkG1;

        let points = points
            .iter()
            .map(|p| ArkG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Arkworks3 {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    #[cfg(feature = "arkworks4")]
    pub fn arkworks4(points: &[[u8; 48]]) -> Self {
        use rust_kzg_arkworks4::kzg_types::ArkG1;

        let points = points
            .iter()
            .map(|p| ArkG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Arkworks4 {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    #[cfg(feature = "arkworks5")]
    pub fn arkworks5(points: &[[u8; 48]]) -> Self {
        use rust_kzg_arkworks5::kzg_types::ArkG1;

        let points = points
            .iter()
            .map(|p| ArkG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Arkworks5 {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    #[cfg(feature = "constantine")]
    pub fn constantine(points: &[[u8; 48]]) -> Self {
        use rust_kzg_constantine::types::g1::CtG1;

        let points = points
            .iter()
            .map(|p| CtG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Constantine {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    #[cfg(feature = "mcl")]
    pub fn mcl(points: &[[u8; 48]]) -> Self {
        use rust_kzg_mcl::types::g1::MclG1;

        let points = points
            .iter()
            .map(|p| MclG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Mcl {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    #[cfg(feature = "zkcrypto")]
    pub fn zkcrypto(points: &[[u8; 48]]) -> Self {
        use rust_kzg_zkcrypto::kzg_types::ZG1;

        let points = points
            .iter()
            .map(|p| ZG1::from_bytes(p))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self::Zkcrypto {
            table: precompute(&points, &[]).unwrap(),
            points,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Blst { .. } => "blst".to_owned(),

            #[cfg(feature = "arkworks3")]
            Self::Arkworks3 { .. } => "arkworks3".to_owned(),

            #[cfg(feature = "arkworks4")]
            Self::Arkworks4 { .. } => "arkworks4".to_owned(),

            #[cfg(feature = "arkworks5")]
            Self::Arkworks5 { .. } => "arkworks5".to_owned(),

            #[cfg(feature = "constantine")]
            Self::Constantine { .. } => "constantine".to_owned(),

            #[cfg(feature = "mcl")]
            Self::Mcl { .. } => "mcl".to_owned(),

            #[cfg(feature = "zkcrypto")]
            Self::Zkcrypto { .. } => "zkcrypto".to_owned(),
        }
    }

    pub fn save_case(&self, points: &[[u8; 48]], scalars: &[[u8; 32]]) -> io::Result<()> {
        let hash: [u8; 32] = {
            let mut hasher = sha2::Sha256::new();

            for p in points {
                hasher.update(p);
            }

            for s in scalars {
                hasher.update(s);
            }

            hasher.finalize().into()
        };

        let failing_case_path =
            PathBuf::from(format!("./failing_case_{}.rs", hex::encode(&hash[0..8])));
        let mut w = File::create(&failing_case_path).expect("failed to save test case");

        let (point_name, scalar_name, fp_name, affine_name, proj_add_affine_name) = match self {
            Self::Blst { .. } => ("FsG1", "FsFr", "FsFp", "FsG1Affine", "FsG1ProjAddAffine"),

            #[cfg(feature = "arkworks3")]
            Self::Arkworks3 { .. } => (
                "ArkG1",
                "ArkFr",
                "ArkFp",
                "ArkG1Affine",
                "ArkG1ProjAddAffine",
            ),

            #[cfg(feature = "arkworks4")]
            Self::Arkworks4 { .. } => (
                "ArkG1",
                "ArkFr",
                "ArkFp",
                "ArkG1Affine",
                "ArkG1ProjAddAffine",
            ),

            #[cfg(feature = "arkworks5")]
            Self::Arkworks5 { .. } => (
                "ArkG1",
                "ArkFr",
                "ArkFp",
                "ArkG1Affine",
                "ArkG1ProjAddAffine",
            ),

            #[cfg(feature = "constantine")]
            Self::Constantine { .. } => ("CtG1", "CtFr", "CtFp", "CtG1Affine", "CtG1ProjAddAffine"),

            #[cfg(feature = "mcl")]
            Self::Mcl { .. } => (
                "MclG1",
                "MclFr",
                "MclFp",
                "MclG1Affine",
                "MclG1ProjAddAffine",
            ),

            #[cfg(feature = "zkcrypto")]
            Self::Zkcrypto { .. } => ("ZG1", "ZFr", "ZFp", "ZG1Affine", "ZG1ProjAddAffine"),
        };

        writeln!(w, "#[test]")?;
        writeln!(w, "fn failing_fuzz_case() {{")?;
        writeln!(
            w,
            "\tuse kzg::{{msm::{{msm_impls::msm, precompute::precompute}}, Fr, G1Mul, G1}};"
        )?;

        match self {
            Self::Blst { .. } => writeln!(w, "\tuse rust_kzg_blst::types::{{fp::FsFp, fr::FsFr, g1::{{FsG1, FsG1Affine, FsG1ProjAddAffine}}}};")?,

            #[cfg(feature = "arkworks3")]
            Self::Arkworks3 { .. } => writeln!(w, "\tuse rust_kzg_arkworks3::kzg_types::{{ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr}};")?,

            #[cfg(feature = "arkworks4")]
            Self::Arkworks4 { .. } => writeln!(w, "\tuse rust_kzg_arkworks4::kzg_types::{{ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr}};")?,

            #[cfg(feature = "arkworks5")]
            Self::Arkworks5 { .. } => writeln!(w, "\tuse rust_kzg_arkworks5::kzg_types::{{ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr}};")?,

            #[cfg(feature = "constantine")]
            Self::Constantine { .. } => writeln!(w, "\tuse rust_kzg_constantine::types::{{fp::CtFp, fr::CtFr, g1::{{CtG1, CtG1Affine, CtG1ProjAddAffine}}}};")?,

            #[cfg(feature = "mcl")]
            Self::Mcl { .. } => writeln!(w, "\tuse rust_kzg_mcl::types::{{fp::MclFp, fr::MclFr, g1::{{MclG1, MclG1Affine, MclG1ProjAddAffine}}}};")?,

            #[cfg(feature = "zkcrypto")]
            Self::Zkcrypto { .. } => writeln!(w, "\tuse rust_kzg_zkcrypto::kzg_types::{{ZG1, ZFp, ZG1Affine, ZG1ProjAddAffine, ZFr}};")?,
        };

        writeln!(w)?;

        writeln!(w, "\tlet points = vec![")?;
        for p in points {
            if FsG1::from_bytes(p).unwrap().is_inf() {
                writeln!(w, "\t\t{point_name}::zero(),")?;
            } else {
                let h = hex::encode(p);
                writeln!(w, "\t\t{point_name}::from_hex(\"0x{h}\").unwrap(),")?;
            }
        }
        writeln!(w, "\t];")?;

        writeln!(w, "")?;

        writeln!(w, "\tlet scalars = vec![")?;
        for s in scalars {
            let scalar = FsFr::from_bytes(s).unwrap();
            if scalar.is_zero() {
                writeln!(w, "\t\t{scalar_name}::zero(),")?;
            } else if scalar.is_one() {
                writeln!(w, "\t\t{scalar_name}::one(),")?;
            } else if FsFr::from_u64(scalar.to_u64_arr()[0]).to_bytes() == *s {
                writeln!(
                    w,
                    "\t\t{scalar_name}::from_u64({}),",
                    scalar.to_u64_arr()[0]
                )?;
            } else {
                let h = hex::encode(s);
                writeln!(w, "\t\t{scalar_name}::from_hex(\"0x{h}\").unwrap(),")?;
            }
        }
        writeln!(w, "\t];")?;

        writeln!(w, "")?;
        writeln!(w, "\tlet mut expected = {}::zero();", point_name)?;
        writeln!(w, "")?;
        writeln!(w, "\tfor (p, s) in points.iter().zip(scalars.iter()) {{")?;
        writeln!(w, "\t\texpected.add_or_dbl_assign(&p.mul(s));")?;
        writeln!(w, "\t}}")?;
        writeln!(w, "")?;
        writeln!(w, "\tlet table = precompute(&points, &[]).unwrap();")?;
        writeln!(w, "\tlet received = msm::<{}, {}, {}, {}, {}>(&points, &scalars, points.len(), table.as_ref());",
            point_name, fp_name, affine_name, proj_add_affine_name, scalar_name)?;
        writeln!(w, "")?;
        writeln!(w, "\tassert!(expected.equals(&received));")?;
        writeln!(w, "}}")?;

        let str = format!(
            "│ Failing test case successfully saved to {} │",
            failing_case_path.display()
        );
        let strlen = str.chars().count();
        println!(
            "\n\n┌{}┐",
            iter::repeat('─').take(strlen - 2).collect::<String>()
        );
        println!("{}", str);
        println!(
            "└{}┘\n\n",
            iter::repeat('─').take(strlen - 2).collect::<String>()
        );

        Ok(())
    }

    pub fn multiply(&self, scalars: &[[u8; 32]]) -> [u8; 48] {
        match self {
            Self::Blst { points, table } => {
                let scalars = scalars
                    .iter()
                    .map(|b| FsFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
            #[cfg(feature = "arkworks3")]
            Self::Arkworks3 { points, table } => {
                use rust_kzg_arkworks3::kzg_types::{
                    ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG1ProjAddAffine,
                };

                let scalars = scalars
                    .iter()
                    .map(|b| ArkFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
            #[cfg(feature = "arkworks4")]
            Self::Arkworks4 { points, table } => {
                use rust_kzg_arkworks4::kzg_types::{
                    ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG1ProjAddAffine,
                };

                let scalars = scalars
                    .iter()
                    .map(|b| ArkFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
            #[cfg(feature = "arkworks5")]
            Self::Arkworks5 { points, table } => {
                use rust_kzg_arkworks5::kzg_types::{
                    ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG1ProjAddAffine,
                };

                let scalars = scalars
                    .iter()
                    .map(|b| ArkFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<ArkG1, ArkFp, ArkG1Affine, ArkG1ProjAddAffine, ArkFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
            #[cfg(feature = "constantine")]
            Self::Constantine { points, table } => {
                use rust_kzg_constantine::types::{
                    fp::CtFp,
                    fr::CtFr,
                    g1::{CtG1, CtG1Affine, CtG1ProjAddAffine},
                };

                let scalars = scalars
                    .iter()
                    .map(|b| CtFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine, CtFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
            #[cfg(feature = "mcl")]
            Self::Mcl { points, table } => {
                use rust_kzg_mcl::types::{
                    fp::MclFp,
                    fr::MclFr,
                    g1::{MclG1, MclG1Affine, MclG1ProjAddAffine},
                };

                let scalars = scalars
                    .iter()
                    .map(|b| MclFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<MclG1, MclFp, MclG1Affine, MclG1ProjAddAffine, MclFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
            #[cfg(feature = "zkcrypto")]
            Self::Zkcrypto { points, table } => {
                use rust_kzg_zkcrypto::kzg_types::{ZFp, ZFr, ZG1Affine, ZG1ProjAddAffine, ZG1};

                let scalars = scalars
                    .iter()
                    .map(|b| ZFr::from_bytes(b))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                let result = msm::<ZG1, ZFp, ZG1Affine, ZG1ProjAddAffine, ZFr>(
                    &points,
                    &scalars,
                    points.len(),
                    table.as_ref(),
                );

                result.to_bytes()
            }
        }
    }
}
