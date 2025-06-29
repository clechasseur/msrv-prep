//! Restores manifests backed up by `cargo msrv-prep` (see `cargo-msrv-prep` crate).

use cargo_msrv_prep::common_args::CommonArgs;
use cargo_msrv_prep::metadata::Metadata;
use cargo_msrv_prep::{maybe_restore_manifest, DEFAULT_MANIFEST_FILE_NAME};
use clap::{Args, Parser};
use log::{debug, info, trace};

fn main() -> cargo_msrv_prep::Result<()> {
    let Cli::MsrvUnprep(args) = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.common.verbose.log_level_filter())
        .init();

    info!("{} started", env!("CARGO_BIN_NAME"));

    unprep_from_msrv(&args)?;

    info!("{} finished", env!("CARGO_BIN_NAME"));
    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cli {
    MsrvUnprep(MsrvUnprepArgs),
}

#[derive(Debug, Args)]
#[command(version, about = "Restores manifests backed up via `cargo msrv-prep`")]
struct MsrvUnprepArgs {
    #[command(flatten)]
    common: CommonArgs,
}

fn unprep_from_msrv(args: &MsrvUnprepArgs) -> cargo_msrv_prep::Result<()> {
    trace!("Entering `unprep_from_msrv` (args: {args:?})");

    let metadata: Metadata = (&args.common).try_into()?;
    debug!("Workspace root: {}", metadata.cargo_metadata.workspace_root);
    debug!("Selected packages: {}", metadata.selected_package_names());

    let mut root_manifest_restored = false;
    for package in &metadata.selected_packages {
        info!("Restoring manifest '{}' (at '{}')", package.name, package.manifest_path);

        maybe_restore_manifest(&package.manifest_path, &args.common.manifest_backup_suffix)?;

        root_manifest_restored = root_manifest_restored
            || package.manifest_path
                == metadata
                    .cargo_metadata
                    .workspace_root
                    .join(DEFAULT_MANIFEST_FILE_NAME);
    }

    if args.common.backup_root_manifest {
        if !root_manifest_restored {
            info!("Restoring root manifest (at '{}')", metadata.cargo_metadata.workspace_root);

            // Note: we do the same assumption here as in `cargo-msrv-prep` (see the corresponding note).
            maybe_restore_manifest(
                &metadata
                    .cargo_metadata
                    .workspace_root
                    .join(DEFAULT_MANIFEST_FILE_NAME),
                &args.common.manifest_backup_suffix,
            )?;
        } else {
            info!("Root manifest already restored; skipping");
        }
    }

    trace!("Exiting `unprep_from_msrv`");
    Ok(())
}
