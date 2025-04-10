use core::{marker::PhantomPinned, pin::Pin, ptr::NonNull};
use std::path::Path;

use crate::types::{
    fft_settings::CtFFTSettings,
    fp::CtFp,
    fr::CtFr,
    g1::{CtG1, CtG1Affine, CtG1ProjAddAffine},
    g2::CtG2,
    kzg_settings::CtKZGSettings as GenericContext,
    poly::CtPoly,
};
use constantine_core::Threadpool as CttThreadpool;
use constantine_ethereum_kzg::EthKzgContext as CttEthKzgContext;
use constantine_sys::{ctt_eth_kzg_status, ctt_eth_trusted_setup_status};
use kzg::KZGSettings;

use super::mixed_eip_4844::verify_kzg_proof_mixed;

struct CttContextInner<'a> {
    ctx: Option<CttEthKzgContext<'a>>,
    pool: Option<CttThreadpool>,
    _pin: PhantomPinned,
}

pub struct CttContext<'a>(Pin<Box<CttContextInner<'a>>>);

impl CttContext<'_> {
    pub fn new(path: &Path) -> Result<Self, ctt_eth_trusted_setup_status> {
        let context = CttEthKzgContext::builder().load_trusted_setup(path)?;

        #[cfg(feature = "parallel")]
        let this = {
            let mut this = Box::new(CttContextInner {
                pool: Some(CttThreadpool::new(
                    constantine_core::hardware::get_num_threads_os(),
                )),
                ctx: None,
                _pin: PhantomPinned,
            });

            this.ctx = Some(
                context
                    .set_threadpool(unsafe { NonNull::from(this.pool.as_ref().unwrap()).as_ref() })
                    .build()?,
            );

            this
        };

        #[cfg(not(feature = "parallel"))]
        let this = {
            Box::new(CttContextInner {
                pool: None,
                ctx: Some(context.build()?),
                _pin: PhantomPinned,
            })
        };

        let pin = Box::into_pin(this);

        Ok(CttContext(pin))
    }

    pub fn blob_to_kzg_commitment(
        &self,
        blob: &[u8; 4096 * 32],
    ) -> Result<[u8; 48], ctt_eth_kzg_status> {
        #[cfg(feature = "parallel")]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .blob_to_kzg_commitment_parallel(blob);

        #[cfg(not(feature = "parallel"))]
        return self.0.ctx.as_ref().unwrap().blob_to_kzg_commitment(blob);
    }

    pub fn compute_kzg_proof(
        &self,
        blob: &[u8; 4096 * 32],
        z_challenge: &[u8; 32],
    ) -> Result<([u8; 48], [u8; 32]), ctt_eth_kzg_status> {
        #[cfg(feature = "parallel")]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .compute_kzg_proof_parallel(blob, z_challenge);

        #[cfg(not(feature = "parallel"))]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .compute_kzg_proof(blob, z_challenge);
    }

    pub fn compute_blob_kzg_proof(
        &self,
        blob: &[u8; 4096 * 32],
        commitment: &[u8; 48],
    ) -> Result<[u8; 48], ctt_eth_kzg_status> {
        #[cfg(feature = "parallel")]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .compute_blob_kzg_proof_parallel(blob, commitment);

        #[cfg(not(feature = "parallel"))]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .compute_blob_kzg_proof(blob, commitment);
    }

    pub fn verify_kzg_proof(
        &self,
        commitment: &[u8; 48],
        z_challenge: &[u8; 32],
        y_eval_at_challenge: &[u8; 32],
        proof: &[u8; 48],
    ) -> Result<bool, ctt_eth_kzg_status> {
        self.0.ctx.as_ref().unwrap().verify_kzg_proof(
            commitment,
            z_challenge,
            y_eval_at_challenge,
            proof,
        )
    }

    pub fn verify_blob_kzg_proof(
        &self,
        blob: &[u8; 4096 * 32],
        commitment: &[u8; 48],
        proof: &[u8; 48],
    ) -> Result<bool, ctt_eth_kzg_status> {
        #[cfg(feature = "parallel")]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .verify_blob_kzg_proof_parallel(blob, commitment, proof);

        #[cfg(not(feature = "parallel"))]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .verify_blob_kzg_proof(blob, commitment, proof);
    }

    pub fn verify_blob_kzg_proof_batch(
        &self,
        blobs: &[[u8; 4096 * 32]],
        commitments: &[[u8; 48]],
        proofs: &[[u8; 48]],
        secure_random_bytes: &[u8; 32],
    ) -> Result<bool, ctt_eth_kzg_status> {
        #[cfg(feature = "parallel")]
        return self
            .0
            .ctx
            .as_ref()
            .unwrap()
            .verify_blob_kzg_proof_batch_parallel(blobs, commitments, proofs, secure_random_bytes);

        #[cfg(not(feature = "parallel"))]
        return self.0.ctx.as_ref().unwrap().verify_blob_kzg_proof_batch(
            blobs,
            commitments,
            proofs,
            secure_random_bytes,
        );
    }
}

// Constantine requires loading from path + doesn't expose underlying secrets, but sometimes required for tests
#[allow(clippy::large_enum_variant)]
pub enum MixedKzgSettings<'a> {
    Constantine(CttContext<'a>),
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

impl MixedKzgSettings<'_> {
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
        Ok(Self::Constantine(
            CttContext::new(path).map_err(|e| e.to_string())?,
        ))
    }
}

impl Default for MixedKzgSettings<'_> {
    fn default() -> Self {
        Self::Generic(GenericContext::default())
    }
}

impl Clone for MixedKzgSettings<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Constantine(_) => panic!("Cannot clone constantine context"),
            Self::Generic(arg0) => Self::Generic(arg0.clone()),
        }
    }
}

// Allow using MixedKzgSettings as KZGSettings stand-in
impl KZGSettings<CtFr, CtG1, CtG2, CtFFTSettings, CtPoly, CtFp, CtG1Affine, CtG1ProjAddAffine>
    for MixedKzgSettings<'_>
{
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
    ) -> Option<
        &kzg::msm::precompute::PrecomputationTable<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>,
    > {
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

    fn get_x_ext_fft_columns(&self) -> &[Vec<CtG1>] {
        match self {
            MixedKzgSettings::Constantine(_) => {
                panic!("Context not in generic format")
            }
            MixedKzgSettings::Generic(generic_context) => generic_context.get_x_ext_fft_columns(),
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
