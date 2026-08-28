use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::{fixture_path, sha256_bytes};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub lab: Lab,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub pragmas: Vec<Pragma>,
    #[serde(default)]
    pub queries: Vec<Query>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lab {
    pub name: String,
    #[serde(default = "default_database")]
    pub database: String,
    pub fixture: PathBuf,
    pub fixture_sha256: String,
    pub sqlite_version: String,
    #[serde(default = "default_warmups")]
    pub warmups: u32,
    #[serde(default = "default_repetitions")]
    pub repetitions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub environment: Environment,
    #[serde(default)]
    pub runner: Runner,
    #[serde(default)]
    pub required_cpu_features: Vec<String>,
    #[serde(default)]
    pub forbidden_cpu_features: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Hardware,
    Virtualized,
    Container,
    Emulator,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Hardware => "hardware",
            Self::Virtualized => "virtualized",
            Self::Container => "container",
            Self::Emulator => "emulator",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    #[default]
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pragma {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub name: String,
    pub sql: String,
    #[serde(default = "default_capture_plan")]
    pub capture_plan: bool,
}

fn default_database() -> String {
    "workload.db".into()
}

fn default_warmups() -> u32 {
    2
}

fn default_repetitions() -> u32 {
    10
}

fn default_capture_plan() -> bool {
    true
}

impl Manifest {
    pub fn validate(&self, manifest_path: &Path) -> Result<()> {
        ensure!(
            self.schema_version == 1,
            "unsupported schema_version {}; expected 1",
            self.schema_version
        );
        ensure!(
            !self.lab.name.trim().is_empty(),
            "lab.name must not be empty"
        );
        ensure!(
            !self.lab.sqlite_version.trim().is_empty(),
            "lab.sqlite_version is a required pin"
        );
        ensure!(
            self.lab.repetitions >= 3,
            "lab.repetitions must be at least 3"
        );
        ensure!(
            self.lab.repetitions <= 10_000,
            "lab.repetitions must not exceed 10000"
        );
        ensure!(
            self.lab.warmups <= 1_000,
            "lab.warmups must not exceed 1000"
        );
        ensure!(
            self.profiles.len() >= 3,
            "declare at least three CPU profiles for release evidence"
        );
        ensure!(!self.queries.is_empty(), "declare at least one query");

        unique_nonempty(self.profiles.iter().map(|item| item.id.as_str()), "profile")?;
        unique_nonempty(self.queries.iter().map(|item| item.name.as_str()), "query")?;
        unique_nonempty(self.pragmas.iter().map(|item| item.name.as_str()), "PRAGMA")?;

        for pragma in &self.pragmas {
            ensure!(
                pragma
                    .name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_'),
                "PRAGMA name {:?} contains unsafe characters",
                pragma.name
            );
            ensure!(
                !pragma.value.contains(';'),
                "PRAGMA {} value must not contain a semicolon",
                pragma.name
            );
        }

        for query in &self.queries {
            ensure!(
                !query.sql.trim().is_empty(),
                "query {} has empty SQL",
                query.name
            );
        }

        for profile in &self.profiles {
            let required: HashSet<_> = profile
                .required_cpu_features
                .iter()
                .map(|value| normalize_feature(value))
                .collect();
            for feature in &profile.forbidden_cpu_features {
                ensure!(
                    !required.contains(&normalize_feature(feature)),
                    "profile {} both requires and forbids CPU feature {}",
                    profile.id,
                    feature
                );
            }
        }

        let fixture = fixture_path(manifest_path, self);
        let bytes = fs::read(&fixture)
            .with_context(|| format!("could not read pinned fixture {}", fixture.display()))?;
        let actual = sha256_bytes(&bytes);
        ensure!(
            actual.eq_ignore_ascii_case(self.lab.fixture_sha256.trim()),
            "fixture SHA-256 mismatch for {}: expected {}, found {}",
            fixture.display(),
            self.lab.fixture_sha256,
            actual
        );

        let runtime_sqlite = rusqlite::version();
        ensure!(
            runtime_sqlite == self.lab.sqlite_version,
            "SQLite version pin mismatch: manifest requires {}, binary contains {}",
            self.lab.sqlite_version,
            runtime_sqlite
        );
        Ok(())
    }

    pub fn profile(&self, id: &str) -> Result<&Profile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown profile {id:?}; declared: {}",
                    self.profiles
                        .iter()
                        .map(|profile| profile.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

fn unique_nonempty<'a>(values: impl Iterator<Item = &'a str>, label: &str) -> Result<()> {
    let mut seen = HashSet::new();
    for value in values {
        if value.trim().is_empty() {
            bail!("{label} name must not be empty");
        }
        if !seen.insert(value) {
            bail!("duplicate {label} name {value:?}");
        }
    }
    Ok(())
}

pub fn normalize_feature(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['.', '-'], "_")
}
