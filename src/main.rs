//! Prepares one or more packages for determining or verifying their crates' MSRV.
//!
//! This cargo command will scan selected packages. For each package, the manifest
//! (e.g. `Cargo.toml`) is loaded and modified as follows:
//!
//! - The manifest's `rust-version` field, if found, is removed
//! - If the manifest has a pinned MSRV dependencies file next to it, its content is
//!   merged with the manifest (see below).
//!
//! If this results in a manifest being modified, then the original is backed up
//! by copying it to a new file next to it. (If a lockfile is also present, it is
//! backed up as well.)
//!
//! Once MSRV has been determined or verified, this process can be undone (e.g. the original
//! manifests restored) by calling `cargo msrv-unprep` (see `cargo-msrv-unprep` crate).
//!
//! # Default values
//!
//! The following default values are used unless overridden via command-line arguments:
//!
//! - Pinned MSRV dependencies file name: `msrv-pins.toml`
//! - Manifest file backup suffix: `.msrv-prep.bak`
//!
//! # Merging MSRV pins
//!
//! The goal of the pinned MSRV dependencies file is to store dependencies that need to be
//! pinned to certain versions in order to properly determine MSRV. This might be required
//! because a package's specified minimally-supported version doesn't actually
//! build (most likely because it is too old, could be successfully built in the past
//! but it no longer works today).
//!
//! The pinned MSRV dependencies file can contain three different types of dependencies:
//!
//! - `dependencies`
//! - `build-dependencies`
//! - Target-specific versions of the two above (e.g. `target.'cfg(unix)'.dependencies`)
//!
//! # Pinned MSRV dependencies file example
//!
//! ```toml
//! [dependencies]
//! foo = "1.0.0"
//!
//! [build-dependencies]
//! bar = "2.0.0"
//!
//! [target.'cfg(windows)'.dependencies]
//! win-specific-baz = "3.0.0"
//!
//! [target.'cfg(unix)'.build-dependencies]
//! unix-specific-build-baz = "4.0.0"
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::fs;

use cargo_msrv_prep::common_args::CommonArgs;
use cargo_msrv_prep::metadata::Metadata;
use cargo_msrv_prep::result::IoErrorContext;
use cargo_msrv_prep::{
    backup_manifest, maybe_merge_msrv_dependencies, remove_rust_version,
    DEFAULT_MANIFEST_FILE_NAME, RUST_VERSION_SPECIFIER,
};
use clap::{crate_name, Args, Parser};
use log::{debug, info, trace};
use toml_edit::DocumentMut;

#[mockall_double::double]
use crate::mockable::fs as mockable_fs;

fn main() -> cargo_msrv_prep::Result<()> {
    let Cli::MsrvPrep(args) = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.common.verbose.log_level_filter())
        .init();

    info!("{} started", crate_name!());

    prep_for_msrv(&args)?;

    info!("{} finished", crate_name!());
    Ok(())
}

mod mockable {
    #[allow(dead_code)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg_attr(test, mockall::automock)]
    pub(super) mod fs {
        use std::fs as real_fs;
        use std::io;
        use std::path::Path;

        #[cfg_attr(test, mockall::concretize)]
        pub fn write<P, C>(path: P, contents: C) -> io::Result<()>
        where
            P: AsRef<Path>,
            C: AsRef<[u8]>,
        {
            real_fs::write(path, contents)
        }
    }
}

/// Default name of TOML file containing pinned crates used when determining/verifying MSRV.
const DEFAULT_MSRV_PINS_FILE_NAME: &str = "msrv-pins.toml";

#[derive(Debug, Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cli {
    MsrvPrep(MsrvPrepArgs),
}

#[derive(Debug, Args)]
#[command(
    version,
    about = "Prepare local manifests for determining/validating MSRV",
    long_about = "Prepare local manifests for determining/validating MSRV\n\
        \n\
        This will perform two operations:\n\
        \n\
        - Remove each package's `rust-version` field (if found)\n\
        - Merge pinned MSRV dependencies in the package's manifest (if found)\n\
        \n\
        To undo changes, run `cargo msrv-unprep`."
)]
struct MsrvPrepArgs {
    #[command(flatten)]
    common: CommonArgs,

    /// Name of TOML file containing pinned dependencies
    #[arg(long, default_value = DEFAULT_MSRV_PINS_FILE_NAME)]
    pub pins_file_name: String,

    /// Skip removing 'rust-version' field
    #[arg(long, default_value_t = false)]
    pub no_remove_rust_version: bool,

    /// Skip merging pinned MSRV dependencies
    #[arg(long, default_value_t = false)]
    pub no_merge_pinned_dependencies: bool,

    /// Overwrite existing manifest backup files
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// Determine if preparation is required without persisting resulting manifests
    ///
    /// To see result, increase verbosity to at least INFO (e.g. `-vv`)
    #[arg(short = 'n', long, default_value_t = false)]
    pub dry_run: bool,
}

fn prep_for_msrv(args: &MsrvPrepArgs) -> cargo_msrv_prep::Result<()> {
    trace!("Entering `prep_for_msrv` (args: {:?})", args);

    let metadata: Metadata = (&args.common).try_into()?;
    debug!("Workspace root: {}", metadata.cargo_metadata.workspace_root);
    debug!("Selected packages: {}", metadata.selected_package_names());

    let mut root_manifest_backed_up = false;
    for package in &metadata.selected_packages {
        info!("Preparing manifest '{}' (at '{}')", package.name, package.manifest_path);

        let manifest_text = fs::read_to_string(&package.manifest_path)
            .with_io_context(|| format!("reading manifest of package {}", package.name))?;
        let mut manifest = manifest_text.parse::<DocumentMut>()?;

        let rust_version_removed = if !args.no_remove_rust_version {
            let removed = remove_rust_version(&mut manifest);

            debug!("'{}' field removed: {}", RUST_VERSION_SPECIFIER, removed);
            removed
        } else {
            info!("Skipping removal of '{}' field", RUST_VERSION_SPECIFIER);
            false
        };

        let msrv_dependencies_merged = if !args.no_merge_pinned_dependencies {
            let merged = maybe_merge_msrv_dependencies(
                &mut manifest,
                &package.manifest_path,
                &args.pins_file_name,
            )?;

            debug!("Pinned MSRV dependencies merged: {}", merged);
            merged
        } else {
            info!("Skipping merging of pinned MSRV dependencies");
            false
        };

        if rust_version_removed || msrv_dependencies_merged {
            if !args.dry_run {
                info!("Manifest for '{}' changed after preparation; persisting", package.name);

                backup_manifest(
                    &package.manifest_path,
                    &args.common.manifest_backup_suffix,
                    args.force,
                )?;
                mockable_fs::write(&package.manifest_path, manifest.to_string()).with_io_context(
                    || format!("saving updated manifest content to '{}'", package.manifest_path),
                )?;
            } else {
                info!(
                    "Manifest for '{}' changed after preparation; not persisting (dry-run mode)",
                    package.name
                );
            }

            root_manifest_backed_up = root_manifest_backed_up
                || package.manifest_path
                    == metadata
                        .cargo_metadata
                        .workspace_root
                        .join(DEFAULT_MANIFEST_FILE_NAME);
        } else {
            info!("Manifest for '{}' not changed after preparation; skipping", package.name);
        }
    }

    if args.common.backup_root_manifest {
        if !root_manifest_backed_up {
            if !args.dry_run {
                info!("Backing up root manifest (at '{}')", metadata.cargo_metadata.workspace_root);

                // Note: this will fail if the root manifest has a non-standard name, but
                // there doesn't seem to be an easy way to fetch the name of the root
                // manifest when it doesn't contain a package itself, so we have no choice.
                backup_manifest(
                    &metadata
                        .cargo_metadata
                        .workspace_root
                        .join(DEFAULT_MANIFEST_FILE_NAME),
                    &args.common.manifest_backup_suffix,
                    args.force,
                )?;
            } else {
                info!("Root manifest needs backup; skipping (dry-run mode)");
            }
        } else {
            info!("Root manifest already backed up; skipping");
        }
    }

    trace!("Exiting `prep_for_msrv`");
    Ok(())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::path::PathBuf;

    use assert_fs::fixture::PathCopy;
    use assert_fs::TempDir;

    use super::*;

    fn project_path(project_name: &str) -> PathBuf {
        [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "cargo-msrv-prep", project_name]
            .iter()
            .collect()
    }

    fn fork_project(project_name: &str) -> TempDir {
        let temp = TempDir::new().unwrap();

        temp.copy_from(project_path(project_name), &["*.rs", "*.toml", "*.lock"])
            .unwrap();

        temp
    }

    mod errors {
        use std::io;

        use assert_fs::fixture::PathChild;
        use assert_matches::assert_matches;
        use cargo_msrv_prep::Error;

        use super::*;

        #[test_log::test]
        fn modified_manifest_write_error() {
            let temp = fork_project("simple_project");

            let Cli::MsrvPrep(args) = Cli::parse_from(
                [
                    "cargo".to_string(),
                    "msrv-prep".to_string(),
                    "--manifest-path".to_string(),
                    temp.child("Cargo.toml").to_string_lossy().to_string(),
                    "-vvvv".to_string(),
                ]
                .iter(),
            );

            let ctx = mockable_fs::write_context();
            ctx.expect().returning(|_, _| {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"))
            });

            assert_matches!(prep_for_msrv(&args), Err(Error::Io { source, .. }) => {
                assert_eq!(io::ErrorKind::PermissionDenied, source.kind());
            });
        }
    }
}
