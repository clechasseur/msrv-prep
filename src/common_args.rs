use clap::Args;
use clap_cargo::{Manifest, Workspace};
use clap_verbosity_flag::Verbosity;

use crate::{DEFAULT_MANIFEST_BACKUP_SUFFIX, DEFAULT_MSRV_PINS_FILE_NAME};

#[derive(Debug, Args)]
pub struct CommonArgs {
    #[command(flatten)]
    pub verbose: Verbosity,

    #[command(flatten)]
    pub manifest: Manifest,
    #[command(flatten)]
    pub workspace: Workspace,

    /// Name of TOML file containing pinned dependencies
    #[arg(long, default_value = DEFAULT_MSRV_PINS_FILE_NAME)]
    pub pins_file_name: String,

    /// Suffix used for manifest backup files
    #[arg(long, default_value = DEFAULT_MANIFEST_BACKUP_SUFFIX)]
    pub manifest_backup_suffix: String,
}
