use clap::Args;
use clap_cargo::{Manifest, Workspace};
use clap_verbosity_flag::Verbosity;

use crate::DEFAULT_MANIFEST_BACKUP_SUFFIX;

#[derive(Debug, Args)]
pub struct CommonArgs {
    #[command(flatten)]
    pub verbose: Verbosity,

    #[command(flatten)]
    pub manifest: Manifest,
    #[command(flatten)]
    pub workspace: Workspace,

    /// Suffix used for manifest backup files
    #[arg(long, default_value = DEFAULT_MANIFEST_BACKUP_SUFFIX)]
    pub manifest_backup_suffix: String,

    /// Always back up the root manifest
    ///
    /// Use to back up the `Cargo.lock` of a workspace without a root package
    #[arg(long, default_value_t = false)]
    pub backup_root_manifest: bool,
}
