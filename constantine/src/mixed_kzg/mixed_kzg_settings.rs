use std::path::Path;

use crate::types::{
    fft_settings::CtFFTSettings,
    fp::CtFp,
    fr::CtFr,
    g1::{CtG1, CtG1Affine},
    g2::CtG2,
    kzg_settings::CtKZGSettings as GenericContext,
    poly::CtPoly,
};
use constantine_core::Threadpool as CttThreadpool;
use constantine_ethereum_kzg::EthKzgContext as CttEthKzgContext;
use constantine_sys::{ctt_eth_kzg_status, ctt_eth_trusted_setup_status};
use kzg::KZGSettings;

use super::mixed_eip_4844::verify_kzg_proof_mixed;

pub struct CttContext {
    pub ctx: CttEthKzgContext,
    pub pool: CttThreadpool,
}

impl CttContext {
    pub fn new(path: &Path) -> Result<Self, String> {
        let res = CttEthKzgContext::load_trusted_setup(path);
        match res {
            Ok(constantine_context) => Ok(Self {
                ctx: constantine_context,
                pool: CttThreadpool::new(get_thr_count()),
            }),
            Err(x) => Err(x.to_string()),
        }
    }
}

fn get_thr_count() -> usize {
    #[cfg(feature = "parallel")]
    return constantine_core::hardware::get_num_threads_os();

    #[cfg(not(feature = "parallel"))]
    return 1;
}

// Constantine requires loading from path + doesn't expose underlying secrets, but sometimes required for tests
#[allow(clippy::large_enum_variant)]
pub enum MixedKzgSettings {
    Constantine(CttContext),
    Generic(GenericContext),
}

pub trait LocalToStr {
    fn to_string(&self) -> String;
}

impl LocalToStr for ctt_eth_trusted_setup_status {
    fn to_string(&self) -> String {
        match self {
            ctt_eth_trusted_setup_status::cttEthTS_InvalidFile => "invalid file".to_owned(),
            ctt_eth_trusted_setup_status::cttEthTS_MissingOrInaccessibleFile => {
                "missing or inaccessible file".to_owned()
            }
            ctt_eth_trusted_setup_status::cttEthTS_Success => "success".to_owned(),
        }
    }
}

impl LocalToStr for ctt_eth_kzg_status {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl MixedKzgSettings {
    pub fn new(
        g1_monomial: &[CtG1],
        g1_lagrange_brp: &[CtG1],
        g2_monomial: &[CtG2],
        fft_settings: &CtFFTSettings,
        cell_size: usize,
    ) -> Result<Self, String> {
        let res = GenericContext::new(
            g1_monomial,
            g1_lagrange_brp,
            g2_monomial,
            fft_settings,
            cell_size,
        );
        match res {
            Ok(generic_context) => Ok(Self::Generic(generic_context)),
            Err(x) => Err(x),
        }
    }

    pub fn new_from_path(path: &Path) -> Result<Self, String> {
        let res = CttEthKzgContext::load_trusted_setup(path);
        match res {
            Ok(constantine_context) => Ok(Self::Constantine(CttContext {
                ctx: constantine_context,
                pool: CttThreadpool::new(get_thr_count()),
            })),
            Err(x) => Err(x.to_string()),
        }
    }
}

impl Default for MixedKzgSettings {
    fn default() -> Self {
        Self::Generic(GenericContext::default())
    }
}

impl Clone for MixedKzgSettings {
    fn clone(&self) -> Self {
        match self {
            Self::Constantine(_) => panic!("Cannot clone constantine context"),
            Self::Generic(arg0) => Self::Generic(arg0.clone()),
        }
    }
}

// Allow using MixedKzgSettings as KZGSettings stand-in
impl KZGSettings<CtFr, CtG1, CtG2, CtFFTSettings, CtPoly, CtFp, CtG1Affine> for MixedKzgSettings {
    fn new(
        g1_monomial: &[CtG1],
        g1_lagrange_brp: &[CtG1],
        g2_monomial: &[CtG2],
        fft_settings: &CtFFTSettings,
        cell_size: usize,
    ) -> Result<Self, String> {
        MixedKzgSettings::new(
            g1_monomial,
            g1_lagrange_brp,
            g2_monomial,
            fft_settings,
            cell_size,
        )
    }

    fn commit_to_poly(&self, p: &CtPoly) -> Result<CtG1, String> {
        match self {
            MixedKzgSettings::Constantine(_) => Err("Context not in generic format".to_string()),
            MixedKzgSettings::Generic(generic_context) => generic_context.commit_to_poly(p),
        }
    }

    fn compute_proof_single(&self, p: &CtPoly, x: &CtFr) -> Result<CtG1, String> {
        match self {
            MixedKzgSettings::Constantine(_) => Err("Context not in generic format".to_string()),
            MixedKzgSettings::Generic(generic_context) => {
                generic_context.compute_proof_single(p, x)
            }
        }
    }

    fn check_proof_single(
        &self,
        com: &CtG1,
        proof: &CtG1,
        x: &CtFr,
        value: &CtFr,
    ) -> Result<bool, String> {
        verify_kzg_proof_mixed(com, x, value, proof, self)
    }

    fn compute_proof_multi(&self, p: &CtPoly, x: &CtFr, n: usize) -> Result<CtG1, String> {
        match self {
            MixedKzgSettings::Constantine(_) => Err("Context not in generic format".to_string()),
            MixedKzgSettings::Generic(generic_context) => {
                generic_context.compute_proof_multi(p, x, n)
            }
        }
    }

    fn check_proof_multi(
        &self,
        com: &CtG1,
        proof: &CtG1,
        x: &CtFr,
        values: &[CtFr],
        n: usize,
    ) -> Result<bool, String> {
        match self {
            MixedKzgSettings::Constantine(_) => Err("Context not in generic format".to_string()),
            MixedKzgSettings::Generic(generic_context) => {
                generic_context.check_proof_multi(com, proof, x, values, n)
            }
        }
    }

    fn get_roots_of_unity_at(&self, i: usize) -> CtFr {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_roots_of_unity_at(i),
        }
    }

    fn get_fft_settings(&self) -> &CtFFTSettings {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_fft_settings(),
        }
    }

    fn get_precomputation(
        &self,
    ) -> Option<&kzg::msm::precompute::PrecomputationTable<CtFr, CtG1, CtFp, CtG1Affine>> {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_precomputation(),
        }
    }

    fn get_g1_monomial(&self) -> &[CtG1] {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_g1_monomial(),
        }
    }

    fn get_g1_lagrange_brp(&self) -> &[CtG1] {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_g1_lagrange_brp(),
        }
    }

    fn get_g2_monomial(&self) -> &[CtG2] {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_g2_monomial(),
        }
    }

    fn get_x_ext_fft_column(&self, index: usize) -> &[CtG1] {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => {
                generic_context.get_x_ext_fft_column(index)
            }
        }
    }

    fn get_cell_size(&self) -> usize {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_cell_size(),
        }
    }
}
