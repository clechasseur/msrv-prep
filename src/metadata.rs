use cargo_metadata::Package;

use crate::common_args::CommonArgs;

pub struct Metadata {
    pub cargo_metadata: cargo_metadata::Metadata,
    pub selected_packages: Vec<Package>,
}

impl TryFrom<&CommonArgs> for Metadata {
    type Error = crate::Error;

    fn try_from(value: &CommonArgs) -> Result<Self, Self::Error> {
        let metadata = value.manifest.metadata().no_deps().exec()?;

        let (selected_packages, _) = value.workspace.partition_packages(&metadata);
        let selected_packages: Vec<_> = selected_packages.into_iter().cloned().collect();

        Ok(Metadata { cargo_metadata: metadata, selected_packages })
    }
}
