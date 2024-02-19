mod toml;

use toml_edit::{table, Item, Table};

use crate::detail::toml::merge_toml;

pub const PACKAGE_SECTION_NAME: &str = "package";
pub const DEPENDENCIES_SECTION_NAME: &str = "dependencies";
pub const BUILD_DEPENDENCIES_SECTION_NAME: &str = "build-dependencies";
pub const TARGET_SECTION_NAME: &str = "target";

pub const RUST_VERSION_SPECIFIER: &str = "rust-version";

pub fn merge_msrv_dependencies(manifest: &mut Table, msrv_dependencies: &Table) -> bool {
    let mut changed = merge_dependencies_sections(manifest, msrv_dependencies);

    if let Some(Item::Table(msrv_target_table)) = msrv_dependencies.get(TARGET_SECTION_NAME) {
        changed = merge_table(manifest, TARGET_SECTION_NAME, msrv_target_table, |dest, src| {
            let mut changed = false;

            for (msrv_key, msrv_value) in src.into_iter() {
                if let Item::Table(msrv_table) = msrv_value {
                    changed = merge_table(dest, msrv_key, msrv_table, |dest, src| {
                        merge_dependencies_sections(dest, src)
                    }) || changed;
                }
            }

            changed
        }) || changed;
    }

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

fn merge_dependencies_sections(manifest: &mut Table, msrv_dependencies: &Table) -> bool {
    let mut changed = false;

    for name in [DEPENDENCIES_SECTION_NAME, BUILD_DEPENDENCIES_SECTION_NAME] {
        if let Some(src_section) = msrv_dependencies.get(name) {
            merge_toml(manifest.entry(name), src_section);
            changed = true;
        }
    }

    changed
}
