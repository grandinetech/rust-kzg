use std::path::Path;

// use crate::
use crate::mixed_kzg_settings::mixed_kzg_settings::MixedKzgSettings;

use crate::types::g1::CtG1Affine;
use crate::types::{fft_settings::CtFFTSettings, fr::CtFr, g1::CtG1, g2::CtG2, poly::CtPoly};

use kzg::eip_4844::{
    blob_to_kzg_commitment_rust, compute_blob_kzg_proof_rust, compute_kzg_proof_rust,
    verify_blob_kzg_proof_batch_rust, verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
    BYTES_PER_BLOB, FIELD_ELEMENTS_PER_BLOB,
};
use kzg::{Fr, G1};

use super::mixed_kzg_settings::LocalToStr;

fn blob_fr_to_byte_inplace(blob: &[CtFr], inplace: &mut [u8; BYTES_PER_BLOB]) -> Option<String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Some("blob length is not equal to FIELD_ELEMENTS_PER_BLOB".to_string());
    }

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        inplace[i * 32..(i + 1) * 32].copy_from_slice(&blob[i].to_bytes());
    }

    None
}

fn blob_fr_to_byte(blob: &[CtFr]) -> Result<[u8; BYTES_PER_BLOB], String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err("blob length is not equal to FIELD_ELEMENTS_PER_BLOB".to_string());
    }

    let mut blob_bytes = [0u8; BYTES_PER_BLOB];
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        blob_bytes[i * 32..(i + 1) * 32].copy_from_slice(&blob[i].to_bytes());
    }

    Ok(blob_bytes)
    // unsafe { Ok(std::mem::transmute(blob.as_ptr() as *const [u8; BYTES_PER_BLOB])) }
}

pub fn load_trusted_setup_filename_mixed(filepath: &str) -> Result<MixedKzgSettings, String> {
    MixedKzgSettings::new_from_path(Path::new(filepath))
}

pub fn blob_to_kzg_commitment_mixed(
    blob: &[CtFr],
    settings: &MixedKzgSettings,
) -> Result<CtG1, String> {
    match settings {
        MixedKzgSettings::Constantine(ctt_context) => {
            let blob_bytes = blob_fr_to_byte(blob)?;

            #[cfg(feature = "parallel")]
            let res = ctt_context
                .ctx
                .blob_to_kzg_commitment_parallel(&ctt_context.pool, &blob_bytes);

            #[cfg(not(feature = "parallel"))]
            let res = ctt_context.ctx.blob_to_kzg_commitment(&blob_bytes);

            match res {
                Ok(commitment) => CtG1::from_bytes(&commitment),
                Err(x) => Err(x.to_string()),
            }
            // return blob_to_kzg_commitment_rust(blob, ctt_context);
        }
        MixedKzgSettings::Generic(generic_context) => {
            return blob_to_kzg_commitment_rust(blob, generic_context);
        }
    }
}

pub fn compute_kzg_proof_mixed(
    blob: &[CtFr],
    z: &CtFr,
    s: &MixedKzgSettings,
) -> Result<(CtG1, CtFr), String> {
    match s {
        MixedKzgSettings::Constantine(ctt_context) => {
            let blob_bytes = blob_fr_to_byte(blob)?;
            unsafe {
                let res = ctt_context
                    .ctx
                    .compute_kzg_proof(&blob_bytes, &z.to_bytes());
                match res {
                    // FIXME: Might not need the from_bytes on Fr here
                    Ok((proof, y)) => Ok((CtG1::from_bytes(&proof)?, CtFr::from_bytes(&y)?)),
                    Err(x) => Err(x.to_string()),
                }
            }
        }
        MixedKzgSettings::Generic(generic_context) => {
            return compute_kzg_proof_rust(blob, z, generic_context);
        }
    }
}

pub fn compute_blob_kzg_proof_mixed(
    blob: &[CtFr],
    commitment: &CtG1,
    ts: &MixedKzgSettings,
) -> Result<CtG1, String> {
    match ts {
        MixedKzgSettings::Constantine(ctt_context) => {
            let blob_bytes = blob_fr_to_byte(blob)?;

            #[cfg(feature = "parallel")]
            let res = ctt_context.ctx.compute_blob_kzg_proof_parallel(
                &ctt_context.pool,
                &blob_bytes,
                &commitment.to_bytes(),
            );

            #[cfg(not(feature = "parallel"))]
            let res = ctt_context
                .ctx
                .compute_blob_kzg_proof(&blob_bytes, &commitment.to_bytes());

            match res {
                Ok(proof) => CtG1::from_bytes(&proof),
                Err(x) => Err(x.to_string()),
            }
        }
        MixedKzgSettings::Generic(generic_context) => {
            return compute_blob_kzg_proof_rust(blob, commitment, generic_context);
        }
    }
}

pub fn verify_kzg_proof_mixed(
    commitment: &CtG1,
    z: &CtFr,
    y: &CtFr,
    proof: &CtG1,
    s: &MixedKzgSettings,
) -> Result<bool, String> {
    match s {
        MixedKzgSettings::Constantine(ctt_context) => {
            let res = unsafe {
                ctt_context.ctx.verify_kzg_proof(
                    &commitment.to_bytes(),
                    &z.to_bytes(),
                    &y.to_bytes(),
                    &proof.to_bytes(),
                )
            };
            match res {
                Ok(x) => Ok(x),
                Err(x) => Err(x.to_string()),
            }
        }
        MixedKzgSettings::Generic(generic_context) => {
            return verify_kzg_proof_rust(commitment, z, y, proof, generic_context);
        }
    }
}

pub fn verify_blob_kzg_proof_mixed(
    blob: &[CtFr],
    commitment_g1: &CtG1,
    proof_g1: &CtG1,
    ts: &MixedKzgSettings,
) -> Result<bool, String> {
    match ts {
        MixedKzgSettings::Constantine(ctt_context) => {
            let blob_bytes = blob_fr_to_byte(blob)?;

            #[cfg(feature = "parallel")]
            let res = ctt_context.ctx.verify_blob_kzg_proof_parallel(
                &ctt_context.pool,
                &blob_bytes,
                &commitment_g1.to_bytes(),
                &proof_g1.to_bytes(),
            );

            #[cfg(not(feature = "parallel"))]
            let res = ctt_context.ctx.verify_blob_kzg_proof(
                &blob_bytes,
                &commitment_g1.to_bytes(),
                &proof_g1.to_bytes(),
            );

            match res {
                Ok(x) => Ok(x),
                Err(x) => Err(x.to_string()),
            }
        }
        MixedKzgSettings::Generic(generic_context) => {
            return verify_blob_kzg_proof_rust(blob, commitment_g1, proof_g1, generic_context);
        }
    }
}

pub fn verify_blob_kzg_proof_batch_mixed(
    blobs: &[Vec<CtFr>],
    commitments_g1: &[CtG1],
    proofs_g1: &[CtG1],
    ts: &MixedKzgSettings,
) -> Result<bool, String> {
    match ts {
        MixedKzgSettings::Constantine(ctt_context) => {
            let mut blobs_storage = vec![[0u8; BYTES_PER_BLOB]; blobs.len()];
            for (i, blob) in blobs.into_iter().enumerate() {
                let res = blob_fr_to_byte_inplace(blob, &mut blobs_storage[i]);
                if res.is_some() {
                    return Err(res.unwrap());
                }
            }
            // let blobs = blobs.iter().map(blob_fr_to_byte_vec).collect::<Result<Vec<_>, _>>()?;

            let commitments = commitments_g1
                .iter()
                .map(|x| x.to_bytes())
                .collect::<Vec<_>>();
            let proofs_g1 = proofs_g1.iter().map(|x| x.to_bytes()).collect::<Vec<_>>();

            let rand_thing = [0u8; 32];

            #[cfg(feature = "parallel")]
            let res = ctt_context.ctx.verify_blob_kzg_proof_batch_parallel(
                &ctt_context.pool,
                blobs_storage.as_slice(),
                commitments.as_slice(),
                proofs_g1.as_slice(),
                &rand_thing,
            );

            #[cfg(not(feature = "parallel"))]
            let res = ctt_context.ctx.verify_blob_kzg_proof_batch(
                blobs_storage.as_slice(),
                commitments.as_slice(),
                proofs_g1.as_slice(),
                &rand_thing,
            );

            match res {
                Ok(x) => Ok(x),
                Err(x) => Err(x.to_string()),
            }
        }
        MixedKzgSettings::Generic(generic_context) => {
            return verify_blob_kzg_proof_batch_rust(
                blobs,
                commitments_g1,
                proofs_g1,
                generic_context,
            );
        }
    }
}
