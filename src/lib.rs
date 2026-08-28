pub mod compare;
pub mod manifest;
pub mod report;
pub mod runner;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

pub use compare::{Comparison, compare_reports};
pub use manifest::{Environment, Manifest, Profile};
pub use report::Report;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn fixture_path(manifest_path: &Path, manifest: &Manifest) -> PathBuf {
    manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.lab.fixture)
}

pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read workload manifest {}", path.display()))?;
    let manifest: Manifest = toml::from_str(&source)
        .with_context(|| format!("could not parse workload manifest {}", path.display()))?;
    manifest.validate(path)?;
    Ok(manifest)
}

pub fn ensure_parent(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("could not create output directory {}", parent.display()))
}

pub fn refuse_overwrite(path: &Path) -> Result<()> {
    if path.exists() {
        bail!(
            "refusing to overwrite {}; choose a new path",
            path.display()
        );
    }
    Ok(())
}
