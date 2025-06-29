mod toml;

use log::{info, trace};
use toml_edit::{table, Item, Table};

use crate::detail::toml::merge_toml;

pub const PACKAGE_SECTION_NAME: &str = "package";
pub const DEPENDENCIES_SECTION_NAME: &str = "dependencies";
pub const BUILD_DEPENDENCIES_SECTION_NAME: &str = "build-dependencies";
pub const TARGET_SECTION_NAME: &str = "target";

pub fn merge_msrv_dependencies(manifest: &mut Table, msrv_dependencies: &Table) -> bool {
    trace!("Entering `merge_msrv_dependencies`");

    let mut changed = merge_dependencies_sections(manifest, msrv_dependencies, None);

    if let Some(Item::Table(msrv_target_table)) = msrv_dependencies.get(TARGET_SECTION_NAME) {
        info!("MSRV dependencies found in '{TARGET_SECTION_NAME}'; merging");

        changed = merge_table(manifest, TARGET_SECTION_NAME, msrv_target_table, |dest, src| {
            let mut changed = false;

            for (msrv_key, msrv_value) in src.into_iter() {
                if let Item::Table(msrv_table) = msrv_value {
                    changed = merge_table(dest, msrv_key, msrv_table, |dest, src| {
                        info!(
                            "MSRV dependencies found in '{TARGET_SECTION_NAME}.{msrv_key}'; merging"
                        );

                        merge_dependencies_sections(
                            dest,
                            src,
                            Some(format!("{TARGET_SECTION_NAME}.{msrv_key}.")),
                        )
                    }) || changed;
                }
            }

            changed
        }) || changed;
    }

    trace!("Exiting `merge_msrv_dependencies` (changed: {changed})");
    changed
}

fn merge_table<F>(destination: &mut Table, key: &str, source: &Table, merge_fn: F) -> bool
where
    F: FnOnce(&mut Table, &Table) -> bool,
{
    match destination.entry(key).or_insert_with(table) {
        Item::Table(dest_table) => merge_fn(dest_table, source),
        dest_item => {
            *dest_item = Item::Table(source.clone());
            true
        },
    }
}

fn merge_dependencies_sections(
    manifest: &mut Table,
    msrv_dependencies: &Table,
    key_prefix: Option<String>,
) -> bool {
    trace!("Entering `merge_dependencies_section` (key_prefix: '{key_prefix:?}'");

    let mut changed = false;
    let key_prefix = key_prefix.as_deref().unwrap_or("");

    for name in [DEPENDENCIES_SECTION_NAME, BUILD_DEPENDENCIES_SECTION_NAME] {
        if let Some(src_section) = msrv_dependencies.get(name) {
            info!("MSRV dependencies found in section '{key_prefix}{name}'; merging");

            merge_toml(manifest.entry(name), src_section);
            changed = true;
        }
    }

    trace!("Exiting `merge_dependencies_section` (changed: {changed})");
    changed
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    mod merge_msrv_dependencies {
        use indoc::indoc;
        use toml_edit::{DocumentMut, ImDocument};

        use super::*;

        #[test_log::test]
        fn test_dependencies_merging() {
            let mut manifest = indoc! {r#"
                [dependencies]
                serde = "1.0.0"
    
                [dev-dependencies]
                indoc = "2.0.0"
    
                [build-dependencies]
                rustc_version = "0.4.0"
            "#}
            .parse::<DocumentMut>()
            .unwrap();

            let msrv_dependencies = indoc! {r#"
                [dependencies]
                thiserror = "1.0.0"
                toml_edit = "0.22.0"
            "#};
            let msrv_dependencies = ImDocument::parse(msrv_dependencies).unwrap();

            assert!(merge_msrv_dependencies(&mut manifest, &msrv_dependencies));

            let expected = indoc! {r#"
                [dependencies]
                serde = "1.0.0"
                thiserror = "1.0.0"
                toml_edit = "0.22.0"
    
                [dev-dependencies]
                indoc = "2.0.0"
    
                [build-dependencies]
                rustc_version = "0.4.0"
            "#};
            assert_eq!(manifest.to_string(), expected);
        }

        #[test_log::test]
        fn test_build_dependencies_merging() {
            let mut manifest = indoc! {r#"
                [dependencies]
                serde = "1.0.0"
    
                [dev-dependencies]
                indoc = "2.0.0"
    
                [build-dependencies]
                rustc_version = "0.4.0"
            "#}
            .parse::<DocumentMut>()
            .unwrap();

            let msrv_dependencies = indoc! {r#"
                [build-dependencies]
                cargo_metadata = "0.18.0"
            "#};
            let msrv_dependencies = ImDocument::parse(msrv_dependencies).unwrap();

            assert!(merge_msrv_dependencies(&mut manifest, &msrv_dependencies));

            let expected = indoc! {r#"
                [dependencies]
                serde = "1.0.0"
    
                [dev-dependencies]
                indoc = "2.0.0"
    
                [build-dependencies]
                rustc_version = "0.4.0"
                cargo_metadata = "0.18.0"
            "#};
            assert_eq!(manifest.to_string(), expected);
        }

        #[test_log::test]
        fn test_target_dependencies_merging() {
            let mut manifest = indoc! {r#"
                [dependencies]
                serde = "1.0.0"
    
                [target.'cfg(windows)'.dependencies]
                win32_api = "1.0.0"
    
                [dev-dependencies]
                indoc = "2.0.0"
    
                [build-dependencies]
                rustc_version = "0.4.0"
    
                [target.'cfg(unix)'.build-dependencies]
                unix_api = "1.0.0"
            "#}
            .parse::<DocumentMut>()
            .unwrap();

            let msrv_dependencies = indoc! {r#"
                [target.'cfg(unix)'.dependencies]
                unix_specific_crate = "1.0.0"
    
                [target.'cfg(unix)'.build-dependencies]
                another_unix_api = "2.0.0"
            "#};
            let msrv_dependencies = ImDocument::parse(msrv_dependencies).unwrap();

            assert!(merge_msrv_dependencies(&mut manifest, &msrv_dependencies));

            let expected = indoc! {r#"
                [dependencies]
                serde = "1.0.0"
                [target.'cfg(unix)'.dependencies]
                unix_specific_crate = "1.0.0"
    
                [target.'cfg(windows)'.dependencies]
                win32_api = "1.0.0"
    
                [dev-dependencies]
                indoc = "2.0.0"
    
                [build-dependencies]
                rustc_version = "0.4.0"
    
                [target.'cfg(unix)'.build-dependencies]
                unix_api = "1.0.0"
                another_unix_api = "2.0.0"
            "#};
            assert_eq!(manifest.to_string(), expected);
        }

        #[test_log::test]
        fn test_type_override() {
            let mut manifest = indoc! {r#"
                target = "what"
            "#}
            .parse::<DocumentMut>()
            .unwrap();

            let msrv_dependencies = indoc! {r#"
                [target.'cfg(unix)'.dependencies]
                unix_specific_crate = "1.0.0"
            "#};
            let msrv_dependencies = ImDocument::parse(msrv_dependencies).unwrap();

            assert!(merge_msrv_dependencies(&mut manifest, &msrv_dependencies));

            let expected = indoc! {r#"
                [target."cfg(unix)".dependencies]
                unix_specific_crate = "1.0.0"
            "#};
            assert_eq!(manifest.to_string(), expected);
        }
    }
}
