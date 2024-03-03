use cargo_metadata::Package;

use crate::common_args::CommonArgs;

pub struct Metadata {
    pub cargo_metadata: cargo_metadata::Metadata,
    pub selected_packages: Vec<Package>,
}

impl Metadata {
    pub fn selected_package_names(&self) -> String {
        self.selected_packages
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl TryFrom<&CommonArgs> for Metadata {
    type Error = crate::Error;

    fn try_from(value: &CommonArgs) -> Result<Self, Self::Error> {
        let metadata = value.manifest.metadata().exec()?;

        let (selected_packages, _) = value.workspace.partition_packages(&metadata);
        let selected_packages: Vec<_> = selected_packages.into_iter().cloned().collect();

        Ok(Metadata { cargo_metadata: metadata, selected_packages })
    }
}
