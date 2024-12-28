use std::path::PathBuf;

use assert_cmd::Command;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::{ChildPath, PathChild, PathCopy};
use assert_fs::TempDir;
use predicates::path::{eq_file, missing};

const MSRV_UNPREP_BIN_EXE: &str = env!("CARGO_BIN_EXE_cargo-msrv-unprep");

fn project_path(project_name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "cargo-msrv-unprep", project_name]
        .iter()
        .collect()
}

fn fork_project(project_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    temp.copy_from(project_path(project_name), &["*.rs", "*.toml", "*.lock", "*.bak"])
        .unwrap();

    temp
}

fn validate_unprep_result(temp_path: &ChildPath, project_path: &ChildPath) {
    let manifest_path = temp_path.child("Cargo.toml");
    if manifest_path.is_file() {
        let mut project_manifest_path = project_path.child("Cargo.toml.msrv-prep.bak");
        if !project_manifest_path.is_file() {
            project_manifest_path = project_path.child("Cargo.toml");
        }

        project_manifest_path.assert(eq_file(manifest_path.path()));
    }
    temp_path
        .child("Cargo.toml.msrv-prep.bak")
        .assert(missing());
    temp_path
        .child("Cargo.lock.msrv-prep.bak")
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

    Command::new(MSRV_UNPREP_BIN_EXE)
        .current_dir(temp.path())
        .arg("msrv-unprep")
        .arg("--workspace")
        .arg("-vvvv")
        .assert()
        .success();

    validate_unprep_result(
        &ChildPath::new(temp.path()),
        &ChildPath::new(project_path(project_name)),
    );
}

mod default_values {
    use super::*;

    macro_rules! unprep_test {
        ($proj_name:ident) => {
            #[test_log::test]
            fn $proj_name() {
                perform_unprep_test(stringify!($proj_name));
            }
        };
    }

    unprep_test!(simple_project);
    unprep_test!(workspace);
    unprep_test!(no_changes);
}

mod custom_values {
    use std::fs;

    use super::*;

    #[test_log::test]
    fn simple_project() {
        let temp = fork_project("simple_project");
        fs::rename(
            temp.child("Cargo.toml.msrv-prep.bak").path(),
            temp.child("Cargo.toml.my-msrv-prep.bak").path(),
        )
        .unwrap();
        fs::rename(
            temp.child("Cargo.lock.msrv-prep.bak").path(),
            temp.child("Cargo.lock.my-msrv-prep.bak").path(),
        )
        .unwrap();

        Command::new(MSRV_UNPREP_BIN_EXE)
            .arg("msrv-unprep")
            .arg("--manifest-path")
            .arg(temp.child("Cargo.toml").to_string_lossy().as_ref())
            .arg("--manifest-backup-suffix")
            .arg(".my-msrv-prep.bak")
            .arg("-vvvv")
            .assert()
            .success();

        ChildPath::new(project_path("simple_project"))
            .child("Cargo.toml.msrv-prep.bak")
            .assert(eq_file(temp.child("Cargo.toml").path()));
        ChildPath::new(project_path("simple_project"))
            .child("Cargo.lock.msrv-prep.bak")
            .assert(eq_file(temp.child("Cargo.lock").path()));
        temp.child("Cargo.toml.my-msrv-prep.bak").assert(missing());
        temp.child("Cargo.lock.my-msrv-prep.bak").assert(missing());
    }

    #[test_log::test]
    fn rootless_workspace_with_root_manifest_backup() {
        let temp = fork_project("rootless_workspace");

        Command::new(MSRV_UNPREP_BIN_EXE)
            .current_dir(temp.path())
            .arg("msrv-unprep")
            .arg("--workspace")
            .arg("--backup-root-manifest")
            .arg("-vvvv")
            .assert()
            .success();

        validate_unprep_result(
            &ChildPath::new(temp.path()),
            &ChildPath::new(project_path("rootless_workspace")),
        );
    }
}
