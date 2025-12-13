use core::marker::PhantomData;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use crate::{G1Affine, G1Fp, G1};

#[allow(dead_code)]
pub struct DiskCache<TG1: G1, TG1Fp: G1Fp, TG1Affine: G1Affine<TG1, TG1Fp>> {
    pub table: Vec<TG1Affine>,
    pub numpoints: usize,

    pub batch_table: Vec<Vec<TG1Affine>>,
    pub batch_numpoints: usize,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
}

pub struct DiskCacheSaveParams<'a, TG1, TG1Affine> {
    pub algorithm: &'a str,
    pub window: usize,
    pub points: &'a [TG1],
    pub matrix: &'a [Vec<TG1>],
    pub table: &'a [TG1Affine],
    pub numpoints: usize,
    pub batch_table: &'a [Vec<TG1Affine>],
    pub batch_numpoints: usize,
    pub contenthash: Option<[u8; 32]>,
}

#[allow(dead_code)]
fn compute_content_hash<TG1: G1, TG1Fp: G1Fp, TG1Affine: G1Affine<TG1, TG1Fp>>(
    points: &[TG1],
    matrix: &[Vec<TG1>],
) -> Result<[u8; 32], String> {
    let mut hasher = Sha256::new();
    for point in points {
        let affine = TG1Affine::into_affine(point);
        hasher
            .write_all(&affine.to_bytes_uncompressed())
            .map_err(|e| format!("{e:?}"))?;
    }
    for row in matrix {
        for point in row {
            let affine = TG1Affine::into_affine(point);
            hasher
                .write_all(&affine.to_bytes_uncompressed())
                .map_err(|e| format!("{e:?}"))?;
        }
    }

    let result: [u8; 32] = hasher.finalize().into();
    Ok(result)
}

#[allow(dead_code)]
impl<TG1: G1, TG1Fp: G1Fp, TG1Affine: G1Affine<TG1, TG1Fp>> DiskCache<TG1, TG1Fp, TG1Affine> {
    /// Function for loading precomputed tables from disk.
    ///
    /// Reads file with name `rust-kzg.{algorithm}.{window}.{type_hash}.cache.bin` from cache
    /// directory. Automatically validates file version & content hash, to avoid
    /// loading invalid precomputations.
    ///
    /// If fails to load precomputation, returns error along with content hash, if
    /// it was computed at that point.
    pub fn load(
        algorithm: &str,
        window: usize,
        points: &[TG1],
        matrix: &[Vec<TG1>],
    ) -> Result<Self, (String, Option<[u8; 32]>)> {
        let cache_dir = dirs::cache_dir();

        let Some(cache_dir) = cache_dir else {
            return Err(("Failed to get cache dir".to_owned(), None));
        };

        // Include type information in cache filename to avoid cross-backend collisions
        let type_hash = {
            let type_name = format!(
                "{}_{}",
                std::any::type_name::<TG1>(),
                std::any::type_name::<TG1Affine>()
            );
            let mut hasher = sha2::Sha256::new();
            hasher.update(type_name.as_bytes());
            let hash_bytes: [u8; 32] = hasher.finalize().into();
            // Use first 8 bytes as hex string for filename
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]
            )
        };

        let cache_path = cache_dir.join(format!(
            "rust-kzg.{algorithm}.{window}.{type_hash}.cache.bin"
        ));
        let cache_file =
            File::open(&cache_path).map_err(|e| (format!("Failed to read cache: {e:?}"), None))?;

        println!("reading msm cache from {cache_path:?}");
        let mut buf_reader = BufReader::new(cache_file);

        let mut buf = [0u8; 96];

        // check file format version
        buf_reader
            .read_exact(&mut buf[0..4])
            .map_err(|e| (format!("Read failure: {e:?}"), None))?;
        if &buf[0..4] != b"kzg1" {
            return Err(("Invalid cache file format".to_owned(), None));
        }

        // check content hash
        let contenthash = compute_content_hash::<TG1, TG1Fp, TG1Affine>(points, matrix)
            .map_err(|e| (format!("Failed to compute content hash: {e}"), None))?;
        buf_reader
            .read_exact(&mut buf[0..32])
            .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
        if contenthash != buf[0..32] {
            return Err(("Invalid content hash".to_owned(), Some(contenthash)));
        }

        buf_reader
            .read_exact(&mut buf[0..8])
            .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
        let numpoints = u64::from_be_bytes(buf[0..8].try_into().unwrap());

        buf_reader
            .read_exact(&mut buf[0..8])
            .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
        let table_size = u64::from_be_bytes(buf[0..8].try_into().unwrap());

        let mut table = Vec::with_capacity(table_size as usize);
        for _ in 0..table_size {
            buf_reader
                .read_exact(&mut buf[0..96])
                .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
            let point = TG1Affine::from_bytes_uncompressed(buf[0..96].try_into().unwrap())
                .map_err(|e| {
                    (
                        format!("Failed to read point from cache, error: {e:?}",),
                        Some(contenthash),
                    )
                })?;
            table.push(point);
        }

        buf_reader
            .read_exact(&mut buf[0..8])
            .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
        let batch_numpoints = u64::from_be_bytes(buf[0..8].try_into().unwrap());

        buf_reader
            .read_exact(&mut buf[0..8])
            .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
        let rows = u64::from_be_bytes(buf[0..8].try_into().unwrap());

        buf_reader
            .read_exact(&mut buf[0..8])
            .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
        let columns = u64::from_be_bytes(buf[0..8].try_into().unwrap());

        let mut batch_table = Vec::with_capacity(rows as usize);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(columns as usize);
            for _ in 0..columns {
                buf_reader
                    .read_exact(&mut buf[0..96])
                    .map_err(|e| (format!("Read failure: {e:?}"), Some(contenthash)))?;
                let point = TG1Affine::from_bytes_uncompressed(buf[0..96].try_into().unwrap())
                    .map_err(|e| {
                        (
                            format!("Failed to read point from cache, error: {e:?}",),
                            Some(contenthash),
                        )
                    })?;
                row.push(point);
            }
            batch_table.push(row);
        }

        Ok(Self {
            table,
            numpoints: numpoints as usize,
            batch_table,
            batch_numpoints: batch_numpoints as usize,
            g1_marker: PhantomData,
            g1_fp_marker: PhantomData,
        })
    }

    pub fn save(params: DiskCacheSaveParams<'_, TG1, TG1Affine>) -> Result<(), String> {
        let DiskCacheSaveParams {
            algorithm,
            window,
            points,
            matrix,
            table,
            numpoints,
            batch_table,
            batch_numpoints,
            contenthash,
        } = params;

        let cache_dir = dirs::cache_dir();

        let Some(cache_dir) = cache_dir else {
            return Err("Failed to get cache dir".to_owned());
        };

        // Include type information in cache filename to avoid cross-backend collisions
        let type_hash = {
            let type_name = format!(
                "{}_{}",
                std::any::type_name::<TG1>(),
                std::any::type_name::<TG1Affine>()
            );
            let mut hasher = sha2::Sha256::new();
            hasher.update(type_name.as_bytes());
            let hash_bytes: [u8; 32] = hasher.finalize().into();
            // Use first 8 bytes as hex string for filename
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]
            )
        };

        let cache_path = cache_dir.join(format!(
            "rust-kzg.{algorithm}.{window}.{type_hash}.cache.bin"
        ));
        let cache_file =
            File::create(&cache_path).map_err(|e| format!("Failed to read cache: {e:?}"))?;

        println!("writing msm cache to {cache_path:?}");

        let mut writer = BufWriter::new(cache_file);

        writer
            .write_all(b"kzg1")
            .map_err(|e| format!("Write failure: {e:?}"))?;

        let contenthash = contenthash
            .map(Ok)
            .unwrap_or_else(|| compute_content_hash::<TG1, TG1Fp, TG1Affine>(points, matrix))?;

        writer
            .write_all(&contenthash)
            .map_err(|e| format!("Write failure: {e:?}"))?;

        writer
            .write_all(&(numpoints as u64).to_be_bytes())
            .map_err(|e| format!("Write failure: {e:?}"))?;

        writer
            .write_all(&(table.len() as u64).to_be_bytes())
            .map_err(|e| format!("Write failure: {e:?}"))?;

        for point in table {
            writer
                .write_all(&point.to_bytes_uncompressed())
                .map_err(|e| format!("Write failure: {e:?}"))?;
        }

        writer
            .write_all(&(batch_numpoints as u64).to_be_bytes())
            .map_err(|e| format!("Write failure: {e:?}"))?;

        writer
            .write_all(&(batch_table.len() as u64).to_be_bytes())
            .map_err(|e| format!("Write failure: {e:?}"))?;

        let columns = batch_table.first().map(|s| s.len()).unwrap_or(0);

        writer
            .write_all(&(columns as u64).to_be_bytes())
            .map_err(|e| format!("Write failure: {e:?}"))?;

        for row in batch_table {
            for point in row {
                writer
                    .write_all(&point.to_bytes_uncompressed())
                    .map_err(|e| format!("Write failure: {e:?}"))?;
            }
        }

        let file = writer
            .into_inner()
            .map_err(|e| format!("Failed to flush: {e:?}"))?;

        file.sync_all()
            .map_err(|e| format!("Failed to sync data to disk: {e:?}"))?;

        Ok(())
    }
}
