//! Cálculo de hashes (MD5, SHA-1, SHA-256).
//!
//! Lectura por tramos de 1 MiB para no cargar archivos enteros en memoria.
//! Nunca se ejecuta el contenido: únicamente se leen bytes.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::models::FileHashes;

const CHUNK_SIZE: usize = 1024 * 1024;

/// Calcula los hashes seleccionados de un archivo de forma streaming.
pub fn compute(
    path: &Path,
    compute_md5: bool,
    compute_sha1: bool,
    compute_sha256: bool,
) -> std::io::Result<FileHashes> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);

    let mut md5_ctx = compute_md5.then(Md5::new);
    let mut sha1_ctx = compute_sha1.then(Sha1::new);
    let mut sha256_ctx = compute_sha256.then(Sha256::new);

    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if let Some(ctx) = md5_ctx.as_mut() {
            ctx.update(&buf[..n]);
        }
        if let Some(ctx) = sha1_ctx.as_mut() {
            ctx.update(&buf[..n]);
        }
        if let Some(ctx) = sha256_ctx.as_mut() {
            ctx.update(&buf[..n]);
        }
    }

    Ok(FileHashes {
        md5: md5_ctx.map(|ctx| hex(ctx.finalize().as_slice())),
        sha1: sha1_ctx.map(|ctx| hex(ctx.finalize().as_slice())),
        sha256: sha256_ctx.map(|ctx| hex(ctx.finalize().as_slice())),
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}
