use toml_edit::{ArrayOfTables, Entry, Item, Table};

/// Merges a source TOML [`Item`] in a destination TOML table (via an [`Entry`]).
///
/// If the destination table does not have an entry for the source item,
/// it is inserted. Otherwise, it is either merged or replaced, as appropriate.
pub fn merge_toml(destination: Entry, source: &Item) {
    match destination {
        Entry::Vacant(vacant_dest) => {
            vacant_dest.insert(source.clone());
        },
        Entry::Occupied(occupied_dest) => {
            let dst_item = occupied_dest.into_mut();
            match source {
                Item::None => *dst_item = Item::None,
                Item::Value(src_value) => *dst_item = Item::Value(src_value.clone()),
                Item::Table(src_table) => match dst_item {
                    Item::Table(dst_table) => merge_toml_table(dst_table, src_table),
                    dst_item => *dst_item = Item::Table(src_table.clone()),
                },
                Item::ArrayOfTables(src_array) => match dst_item {
                    Item::ArrayOfTables(dst_array) => merge_toml_array(dst_array, src_array),
                    dst_item => *dst_item = Item::ArrayOfTables(src_array.clone()),
                },
            }
        },
    }
}

fn merge_toml_table(destination: &mut Table, source: &Table) {
    for (src_key, src_value) in source {
        merge_toml(destination.entry(src_key), src_value);
    }
}

fn merge_toml_array(destination: &mut ArrayOfTables, source: &ArrayOfTables) {
    for src_table in source {
        destination.push(src_table.clone());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use indoc::indoc;
    use toml_edit::{value, Datetime, Document, DocumentMut, Value};

    use super::*;

    #[test]
    fn test_simple_items() {
        let mut destination = indoc! {r#"
            [table]
            was_already_there = "hello!" # comment
        "#}
        .parse::<DocumentMut>()
        .unwrap();
        let destination_table = destination["table"].as_table_mut().unwrap();

        merge_toml(destination_table.entry("life"), &value(42));
        merge_toml(destination_table.entry("hangar"), &value("23"));
        merge_toml(destination_table.entry("cool"), &value(true));
        merge_toml(destination_table.entry("monay"), &value(101.2));
        merge_toml(
            destination_table.entry("small_step"),
            &value("1969-07-20T20:05:00Z".parse::<Datetime>().unwrap()),
        );
        merge_toml(destination_table.entry("algebra"), &value(Value::from_iter([7, 11])));
        merge_toml(
            destination_table.entry("recursion"),
            &value(Value::from_iter([("did_you_mean", "recursion")])),
        );

        let expected = indoc! {r#"
            [table]
            was_already_there = "hello!" # comment
            life = 42
            hangar = "23"
            cool = true
            monay = 101.2
            small_step = 1969-07-20T20:05:00Z
            algebra = [7, 11]
            recursion = { did_you_mean = "recursion" }
        "#};
        assert_eq!(destination.to_string(), expected);
    }

    #[test]
    fn test_table() {
        let mut destination = indoc! {r#"
            [table]
            was_already_there = "hello!" # comment
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        let source = indoc! {r#"
            [table]
            life = 42
            hangar = "23"
        "#};
        let source = Document::parse(source).unwrap();

        merge_toml(destination.entry("table"), &source["table"]);

        let expected = indoc! {r#"
            [table]
            was_already_there = "hello!" # comment
            life = 42
            hangar = "23"
        "#};
        assert_eq!(destination.to_string(), expected);
    }

    #[test]
    fn test_array_of_tables() {
        let mut destination = indoc! {r#"
            [[tables]]
            was_already_there = "hello!" # comment
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        let source = indoc! {r#"
            [[tables]]
            life = 42
            [[tables]]
            hangar = "23"
            cool = true
        "#};
        let source = Document::parse(source).unwrap();

        merge_toml(destination.entry("tables"), &source["tables"]);

        let expected = indoc! {r#"
            [[tables]]
            was_already_there = "hello!" # comment
            [[tables]]
            life = 42
            [[tables]]
            hangar = "23"
            cool = true
        "#};
        assert_eq!(destination.to_string(), expected);
    }

    #[test]
    fn test_replacement() {
        let mut destination = indoc! {r#"
            table = "what"
            tables = "how"
            will_be_overwritten = "alas, poor Yorick"
            will_be_removed = "salut"
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        let source = indoc! {r#"
            will_be_overwritten = "hello, world!"

            [table]
            life = 42
            hangar = "23"

            [[tables]]
            always = [7, 11]
        "#};
        let source = Document::parse(source).unwrap();

        merge_toml(destination.entry("table"), &source["table"]);
        merge_toml(destination.entry("tables"), &source["tables"]);
        merge_toml(destination.entry("will_be_overwritten"), &source["will_be_overwritten"]);
        merge_toml(destination.entry("will_be_removed"), &Item::None);

        let expected = indoc! {r#"
            will_be_overwritten = "hello, world!"

            [table ]
            life = 42
            hangar = "23"

            [[tables ]]
            always = [7, 11]
        "#};
        assert_eq!(destination.to_string(), expected);
    }

    #[test]
    fn test_recursive_tables() {
        let mut destination = indoc! {r#"
            [dependencies]
            foo = "1.0.0"

            [target.'cfg(unix)'.dependencies]
            unix_foo = "1.0.0"

            [target.'cfg(windows)'.dependencies]
            win32_foo = "1.0.0"
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        let source = indoc! {r#"
            [dependencies]
            bar = "1.0.0"
            baz = "1.0.0"

            [target.'cfg(unix)'.dependencies]
            unix_bar = "1.0.0"

            [target.'cfg(beos)'.dependencies]
            beos_foo = "1.0.0"
        "#};
        let source = Document::parse(source).unwrap();

        merge_toml(destination.entry("dependencies"), &source["dependencies"]);
        merge_toml(destination.entry("target"), &source["target"]);

        let expected = indoc! {r#"
            [dependencies]
            foo = "1.0.0"
            bar = "1.0.0"
            baz = "1.0.0"

            [target.'cfg(unix)'.dependencies]
            unix_foo = "1.0.0"
            unix_bar = "1.0.0"

            [target.'cfg(windows)'.dependencies]
            win32_foo = "1.0.0"

            [target."cfg(beos)".dependencies]
            beos_foo = "1.0.0"
        "#};
        assert_eq!(destination.to_string(), expected);
    }
}
