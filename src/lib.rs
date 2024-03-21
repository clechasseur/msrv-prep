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
pub(crate) mod mockable;
pub mod result;

use std::fs;

use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use log::{debug, error, info, trace, warn};
pub use result::Error;
pub use result::Result;
use toml_edit::{ImDocument, Item, Table};

use crate::detail::{merge_msrv_dependencies, PACKAGE_SECTION_NAME};
#[mockall_double::double]
use crate::mockable::fs as mockable_fs;
use crate::result::IoErrorContext;

/// Default suffix used to backup manifest files before determining/verifying MSRV.
pub const DEFAULT_MANIFEST_BACKUP_SUFFIX: &str = ".msrv-prep.bak";

/// Field in the `package` section of a manifest that stores the package's MSRV.
pub const RUST_VERSION_SPECIFIER: &str = "rust-version";

/// Extension used for lockfiles.
pub const LOCKFILE_EXT: &str = "lock";

/// Removes the `rust-version` field from a Cargo manifest's
/// `package` section, if present.
///
/// Returns `true` if the manifest was modified.
pub fn remove_rust_version(manifest: &mut Table) -> bool {
    trace!("Entering `remove_rust_version`");

    let changed = match manifest.get_mut(PACKAGE_SECTION_NAME) {
        Some(Item::Table(package)) => {
            info!(
                "'package' section found in manifest, removing '{}' field",
                RUST_VERSION_SPECIFIER
            );

            package.remove(RUST_VERSION_SPECIFIER).is_some()
        },
        _ => false,
    };

    trace!("Exiting `remove_rust_version` (changed: {})", changed);
    changed
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
    trace!(
        "Entering `maybe_merge_msrv_dependencies` (manifest_path: '{}', pins_file_name: '{}')",
        manifest_path,
        pins_file_name
    );
    let mut changed = false;

    let pins_file_path = manifest_path.parent().map(|par| par.join(pins_file_name));

    if let Some(pins_file_path) = pins_file_path {
        debug!("Pinned MSRV dependencies file path: {}", pins_file_path);

        if pins_file_path.is_file() {
            info!(
                "Pinned MSRV dependencies file found at '{}'; merging with manifest at '{}'",
                pins_file_path, manifest_path
            );

            let pins_file_text = fs::read_to_string(&pins_file_path)
                .with_io_context(|| format!("reading MSRV pins file '{}'", pins_file_path))?;
            let pins_file = ImDocument::parse(pins_file_text)?;

            changed = merge_msrv_dependencies(manifest, &pins_file);
        }
    } else {
        warn!("Pinned MSRV dependencies file path could not be determined; skipping");
    }

    trace!("Exiting `maybe_merge_msrv_dependencies` (changed: {})", changed);
    Ok(changed)
}

/// Backs up a manifest file by copying it to a new file next to it.
///
/// The new file's name is the same as the manifest, with the given backup suffix appended.
///
/// If a lockfile exists next to the manifest, it is also backed up in a similar manner.
pub fn backup_manifest(manifest_path: &Utf8Path, backup_suffix: &str, force: bool) -> Result<()> {
    trace!(
        "Entering `backup_manifest` (manifest_path: '{}', backup_suffix: '{}', force: {})",
        manifest_path,
        backup_suffix,
        force,
    );

    let lockfile_path = manifest_path.with_extension(LOCKFILE_EXT);

    let manifest_backup_path = get_backup_path(manifest_path, backup_suffix)?;
    let lockfile_backup_path = get_backup_path(&lockfile_path, backup_suffix)?;

    validate_backup_file(&manifest_backup_path, force)?;
    if lockfile_path.is_file() {
        validate_backup_file(&lockfile_backup_path, force)?;
    }

    backup_file(manifest_path, &manifest_backup_path)?;
    if lockfile_path.is_file() {
        backup_file(&lockfile_path, &lockfile_backup_path)?;
    }

    trace!("Exiting `backup_manifest`");
    Ok(())
}

/// If a backup manifest exists next to the given manifest, restores it.
///
/// The backup manifest must've been created by calling [`backup_manifest`]
/// (passing it the same `backup_suffix` value).
///
/// If a lockfile was also backed up next to the manifest, it is also restored.
pub fn maybe_restore_manifest(manifest_path: &Utf8Path, backup_suffix: &str) -> Result<()> {
    trace!(
        "Entering `maybe_restore_manifest` (manifest_path: '{}', backup_suffix: '{}')",
        manifest_path,
        backup_suffix
    );

    let lockfile_path = manifest_path.with_extension(LOCKFILE_EXT);

    maybe_restore_file(manifest_path, backup_suffix)?;

    if lockfile_path.is_file() {
        maybe_restore_file(&lockfile_path, backup_suffix)?;
    }

    trace!("Exiting `maybe_restore_manifest`");
    Ok(())
}

fn maybe_restore_file(file_path: &Utf8Path, backup_suffix: &str) -> Result<()> {
    trace!(
        "Entering `maybe_restore_file` (file_path: '{}', backup_suffix: '{}')",
        file_path,
        backup_suffix
    );

    let backup_path = get_backup_path(file_path, backup_suffix)?;
    debug!("Backup path: {}", backup_path);

    if backup_path.is_file() {
        info!("Backup file found at '{}'; restoring to '{}'", backup_path, file_path);

        mockable_fs::rename(&backup_path, file_path).with_io_context(|| {
            format!("restoring backup from '{}' to '{}'", backup_path, file_path)
        })?;
    }

    trace!("Exiting `maybe_restore_file`");
    Ok(())
}

fn get_backup_path(file_path: &Utf8Path, backup_suffix: &str) -> Result<Utf8PathBuf> {
    file_path
        .file_name()
        .map(|name| name.to_string() + backup_suffix)
        .and_then(|name| file_path.parent().map(|par| par.join(name)))
        .ok_or_else(|| Error::InvalidPath(file_path.into()))
}

fn validate_backup_file(backup_path: &Utf8Path, force: bool) -> Result<()> {
    match (backup_path.is_file(), force) {
        (true, true) => {
            info!(
                "Backup file already exists at '{}'; will be overwritten (forced backup)",
                backup_path
            );
            Ok(())
        },
        (true, false) => {
            error!("Backup file already exists at '{}'; aborting", backup_path);

            Err(Error::BackupFileAlreadyExists(backup_path.into()))
        },
        (false, _) => Ok(()),
    }
}

fn backup_file(file_path: &Utf8Path, backup_path: &Utf8Path) -> Result<()> {
    info!("Backing up '{}' to '{}'", file_path, backup_path);
    mockable_fs::copy(file_path, backup_path)
        .map(|_| ())
        .with_io_context(|| format!("backing up '{}' to '{}'", file_path, backup_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod remove_rust_version {
        use indoc::indoc;
        use toml_edit::DocumentMut;

        use super::*;

        #[test_log::test]
        fn no_rust_version() {
            let manifest_text = indoc! {r#"
                [table]
                hangar = 23
            "#};
            let mut manifest = manifest_text.parse::<DocumentMut>().unwrap();

            let changed = remove_rust_version(&mut manifest);

            assert!(!changed);
            assert_eq!(manifest_text, manifest.to_string());
        }
    }

    mod maybe_merge_msrv_dependencies {
        use assert_matches::assert_matches;

        use super::*;

        #[test_log::test]
        fn skip_parent_path() {
            let changed =
                maybe_merge_msrv_dependencies(&mut Table::new(), "".into(), "msrv-pins.toml");

            assert_matches!(changed, Ok(false));
        }
    }

    mod backup_manifest {
        use super::*;

        mod errors {
            use std::io;

            use assert_matches::assert_matches;

            use super::*;

            #[test_log::test]
            fn backup_copy_error() {
                let project_path: Utf8PathBuf = [
                    env!("CARGO_MANIFEST_DIR"),
                    "resources",
                    "tests",
                    "cargo-msrv-prep",
                    "simple_project",
                ]
                .iter()
                .collect();

                let ctx = mockable_fs::copy_context();
                ctx.expect().returning(|_, _| {
                    Err(io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"))
                });

                let result = backup_manifest(
                    &project_path.join("Cargo.toml"),
                    DEFAULT_MANIFEST_BACKUP_SUFFIX,
                    true,
                );
                assert_matches!(result, Err(Error::Io { source, .. }) => {
                    assert_eq!(io::ErrorKind::PermissionDenied, source.kind());
                });
            }
        }
    }

    mod maybe_restore_manifest {
        use super::*;

        mod errors {
            use std::io;

            use assert_matches::assert_matches;

            use super::*;

            #[test_log::test]
            fn backup_rename_error() {
                let project_path: Utf8PathBuf = [
                    env!("CARGO_MANIFEST_DIR"),
                    "resources",
                    "tests",
                    "cargo-msrv-unprep",
                    "simple_project",
                ]
                .iter()
                .collect();

                let ctx = mockable_fs::rename_context();
                ctx.expect().returning(|_, _| {
                    Err(io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"))
                });

                let result = maybe_restore_manifest(
                    &project_path.join("Cargo.toml"),
                    DEFAULT_MANIFEST_BACKUP_SUFFIX,
                );
                assert_matches!(result, Err(Error::Io { source, .. }) => {
                    assert_eq!(io::ErrorKind::PermissionDenied, source.kind());
                });
            }
        }
    }
}
