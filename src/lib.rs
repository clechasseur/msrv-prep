//! Cargo subcommand useful to prepare for determining/verifying a crate's MSRV.
//!
//! This crate is a library used by the two `cargo` commands provided:
//!
//! - `cargo-msrv-prep`
//! - `cargo-msrv-unprep`
//!
//! This library is not meant for external use and makes no guarantee on API stability.
//!
//! To install `cargo-msrv-prep`, see [the project's GitHub page](https://github.com/clechasseur/msrv-prep).

pub mod common_args;
mod detail;
pub mod metadata;
pub mod result;

use std::fs;

use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::Package;
pub use result::Error;
pub use result::Result;
use toml_edit::{Document, Item, Table};

use crate::detail::{merge_msrv_dependencies, PACKAGE_SECTION_NAME, RUST_VERSION_SPECIFIER};
use crate::result::IoErrorContext;

/// Default suffix used to backup manifest files before determining/verifying MSRV.
pub const DEFAULT_MANIFEST_BACKUP_SUFFIX: &str = ".msrv-prep.bak";

/// Restores manifests modified by `cargo-msrv-prep`.
///
/// This will scan the provided packages and look for backed-up original manifest
/// files; any found is restored to its original version.
pub fn unprep_from_msrv(packages: Vec<&Package>, backup_suffix: &str) -> Result<()> {
    for package in packages {
        maybe_restore_manifest(&package.manifest_path, backup_suffix)?;
    }

    Ok(())
}

/// Removes the `rust-version` field from a Cargo manifest's
/// `package` section, if present.
///
/// Returns `true` if the manifest was modified.
pub fn remove_rust_version(manifest: &mut Table) -> bool {
    match manifest.get_mut(PACKAGE_SECTION_NAME) {
        Some(Item::Table(package)) => package.remove(RUST_VERSION_SPECIFIER).is_some(),
        _ => false,
    }
}

/// Merges optional MSRV dependencies in a Cargo manifest if they exist.
///
/// The optional pinned MSRV dependencies need to be stored in a file next to the Cargo manifest.
///
/// Returns `Ok(true)` if the manifest was modified.
pub fn maybe_merge_msrv_dependencies(
    manifest: &mut Table,
    manifest_path: &Utf8Path,
    pins_file_name: &str,
) -> Result<bool> {
    let pins_file_path = manifest_path.parent().map(|par| par.join(pins_file_name));

    if let Some(pins_file_path) = pins_file_path {
        if pins_file_path.is_file() {
            let pins_file_text = fs::read_to_string(&pins_file_path)
                .with_io_context(|| format!("reading MSRV pins file {}", pins_file_path))?;
            let pins_file = pins_file_text.parse::<Document>()?;

            return Ok(merge_msrv_dependencies(manifest, &pins_file));
        }
    }

    Ok(false)
}

/// Backs up a manifest file by copying it to a new file next to it.
///
/// The new file's name is the same as the manifest, with the given backup suffix appended.
pub fn backup_manifest(manifest_path: &Utf8Path, backup_suffix: &str) -> Result<()> {
    let backup_path = get_backup_path(manifest_path, backup_suffix)?;

    if backup_path.exists() {
        return Err(Error::BackupManifestAlreadyExists(backup_path));
    }

    fs::copy(manifest_path, &backup_path)
        .map(|_| ())
        .with_io_context(|| {
            format!("backing up manifest at '{}' to '{}'", manifest_path, backup_path)
        })
}

/// If a backup manifest exists next to the given manifest, restores it.
///
/// The backup manifest must've been created by calling [`backup_manifest`]
/// (passing it the same `backup_suffix` value).
pub fn maybe_restore_manifest(manifest_path: &Utf8Path, backup_suffix: &str) -> Result<()> {
    let backup_path = get_backup_path(manifest_path, backup_suffix)?;

    if backup_path.is_file() {
        fs::rename(&backup_path, manifest_path).with_io_context(|| {
            format!("restoring manifest backup from '{}' to '{}'", backup_path, manifest_path)
        })?;
    }

    Ok(())
}

fn get_backup_path(manifest_path: &Utf8Path, backup_suffix: &str) -> Result<Utf8PathBuf> {
    manifest_path
        .file_name()
        .map(|mfn| mfn.to_string() + backup_suffix)
        .and_then(|bfn| manifest_path.parent().map(|par| par.join(bfn)))
        .ok_or_else(|| Error::InvalidManifestPath(manifest_path.into()))
}
