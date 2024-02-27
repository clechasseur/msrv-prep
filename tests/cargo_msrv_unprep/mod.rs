use std::path::PathBuf;

use assert_cmd::Command;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::{ChildPath, PathChild, PathCopy};
use assert_fs::TempDir;
use predicates::path::{eq_file, missing};

const MSRV_UNPREP_BIN_NAME: &str = env!("CARGO_BIN_EXE_cargo-msrv-unprep");

fn project_path(project_name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "cargo-msrv-unprep", project_name]
        .iter()
        .collect()
}

fn fork_project(project_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    temp.copy_from(project_path(project_name), &["*.rs", "*.toml"])
        .unwrap();

    temp
}

fn validate_unprep_result(temp_path: &ChildPath, project_path: &ChildPath) {
    let manifest_path = temp_path.child("Cargo.toml");
    if manifest_path.is_file() {
        project_path
            .child("Cargo.toml")
            .assert(eq_file(manifest_path.path()));
    }
    temp_path
        .child("Cargo.toml.msrv-prep.bak")
        .assert(missing());

    for child in temp_path.read_dir().unwrap() {
        let child = child.unwrap();

        if child.file_type().unwrap().is_dir() {
            let child_name = child.file_name();
            validate_unprep_result(&temp_path.child(&child_name), &project_path.child(&child_name));
        }
    }
}

fn perform_unprep_test(project_name: &str) {
    let temp = fork_project(project_name);

    let mut cmd = Command::cargo_bin(MSRV_UNPREP_BIN_NAME).unwrap();
    let assert = cmd
        .current_dir(temp.path())
        .arg("msrv-unprep")
        .arg("--workspace")
        .arg("-vvvv")
        .assert();

    assert.success();

    validate_unprep_result(
        &ChildPath::new(temp.path()),
        &ChildPath::new(project_path(project_name)),
    );
}

macro_rules! unprep_test {
    ($proj_name:ident) => {
        mod $proj_name {
            use super::*;

            #[test_log::test]
            fn unprep() {
                perform_unprep_test(stringify!($proj_name));
            }
        }
    };
}

unprep_test!(simple_project);
unprep_test!(workspace);
unprep_test!(no_changes);
