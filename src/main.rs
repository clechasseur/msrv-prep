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
//! by copying it to a new file next to it.
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

use std::fs;

use cargo_msrv_prep::common_args::CommonArgs;
use cargo_msrv_prep::metadata::Metadata;
use cargo_msrv_prep::result::IoErrorContext;
use cargo_msrv_prep::{backup_manifest, maybe_merge_msrv_dependencies, remove_rust_version};
use clap::{Args, Parser};
use toml_edit::Document;

fn main() -> cargo_msrv_prep::Result<()> {
    let Cli::MsrvPrep(args) = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.common_args.verbose.log_level_filter())
        .init();

    prep_for_msrv(&args.common_args)
}

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
    common_args: CommonArgs,
}

fn prep_for_msrv(args: &CommonArgs) -> cargo_msrv_prep::Result<()> {
    let metadata: Metadata = args.try_into()?;

    for package in &metadata.selected_packages {
        let manifest_text = fs::read_to_string(&package.manifest_path)
            .with_io_context(|| format!("reading manifest of package {}", package.name))?;
        let mut manifest = manifest_text.parse::<Document>()?;
        let changed = remove_rust_version(&mut manifest)
            || maybe_merge_msrv_dependencies(
                &mut manifest,
                &package.manifest_path,
                &args.pins_file_name,
            )?;

        if changed {
            backup_manifest(&package.manifest_path, &args.manifest_backup_suffix)?;
            fs::write(&package.manifest_path, &manifest.to_string()).with_io_context(|| {
                format!("saving updated manifest content to '{}'", package.manifest_path)
            })?;
        }
    }

    Ok(())
}
