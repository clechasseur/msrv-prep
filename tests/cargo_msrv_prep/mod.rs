use std::fs;
use std::path::{Path, PathBuf};

use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;
use toml::Table;

const MSRV_PREP_BIN_NAME: &str = env!("CARGO_BIN_EXE_cargo-msrv-prep");

fn project_path(project_name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "cargo-msrv-prep", project_name]
        .iter()
        .collect()
}

fn fork_project(project_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    temp.copy_from(project_path(project_name), &["*.rs", "*.toml"])
        .unwrap();

    temp
}

fn toml_files_equal<A, B>(file_a: A, file_b: B) -> bool
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let toml_a = fs::read_to_string(file_a).unwrap();
    let toml_b = fs::read_to_string(file_b).unwrap();

    let toml_a = toml_a.parse::<Table>().unwrap();
    let toml_b = toml_b.parse::<Table>().unwrap();

    toml_a == toml_b
}

mod simple_project {
    use assert_cmd::Command;
    use assert_fs::assert::PathAssert;
    use assert_fs::fixture::PathChild;
    use predicates::path::missing;

    use super::*;

    #[test_log::test]
    fn all() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("expected").child("all.toml").path(),
            temp.child("Cargo.toml").path()
        ));
    }

    #[test_log::test]
    fn no_remove_rust_version() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--no-remove-rust-version")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("expected")
                .child("no-remove-rust-version.toml")
                .path(),
            temp.child("Cargo.toml").path()
        ));
    }

    #[test_log::test]
    fn no_merge_pinned_dependencies() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--no-merge-pinned-dependencies")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("expected")
                .child("no-merge-pinned-dependencies.toml")
                .path(),
            temp.child("Cargo.toml").path()
        ));
    }

    #[test_log::test]
    fn dry_run() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--dry-run")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("Cargo.toml").path(),
            project_path("simple_project").join("Cargo.toml")
        ));
        temp.child("Cargo.toml.msrv-prep.bak").assert(missing());
    }

    #[test_log::test]
    fn effectively_a_dry_run() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--no-remove-rust-version")
            .arg("--no-merge-pinned-dependencies")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("Cargo.toml").path(),
            project_path("simple_project").join("Cargo.toml")
        ));
        temp.child("Cargo.toml.msrv-prep.bak").assert(missing());
    }

    #[test_log::test]
    fn fail_because_backup_manifest_already_exists() {
        let temp = fork_project("simple_project");
        fs::write(temp.child("Cargo.toml.msrv-prep.bak"), b"").unwrap();

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("-vvvv")
            .assert();

        assert.failure();

        assert_eq!("", fs::read_to_string(temp.child("Cargo.toml.msrv-prep.bak").path()).unwrap());
    }

    #[test_log::test]
    fn overwrite_existing_manifest_backup() {
        let temp = fork_project("simple_project");
        fs::write(temp.child("Cargo.toml.msrv-prep.bak"), b"").unwrap();

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--force")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("expected").child("all.toml").path(),
            temp.child("Cargo.toml").path()
        ));
    }
}

mod workspace {
    use assert_cmd::Command;
    use assert_fs::assert::PathAssert;
    use assert_fs::fixture::PathChild;
    use predicates::path::missing;

    use super::*;

    fn validate_workspace_result<'a, 'b, C, U>(temp: &TempDir, changed: C, unchanged: U)
    where
        C: IntoIterator<Item = &'a str>,
        U: IntoIterator<Item = &'b str>,
    {
        let project_path = project_path("workspace");

        for package in changed {
            assert!(toml_files_equal(
                temp.child(package).child("expected.toml").path(),
                temp.child(package).child("Cargo.toml").path()
            ));
            assert!(toml_files_equal(
                temp.child(package).child("Cargo.toml.msrv-prep.bak").path(),
                project_path.join(package).join("Cargo.toml")
            ));
        }

        for package in unchanged {
            assert!(toml_files_equal(
                temp.child(package).child("Cargo.toml").path(),
                project_path.join(package).join("Cargo.toml")
            ));
            temp.child(package)
                .child("Cargo.toml.msrv-prep.bak")
                .assert(missing());
        }
    }

    #[test_log::test]
    fn all() {
        let temp = fork_project("workspace");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--workspace")
            .arg("-vvvv")
            .assert();

        assert.success();

        validate_workspace_result(&temp, ["", "member_a", "member_b", "member_c"], []);
    }

    mod specific_packages_tests {
        use super::*;

        fn test_with_package(package: &str, package_dir: &str) {
            let temp = fork_project("workspace");

            let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
            let assert = cmd
                .current_dir(temp.path())
                .arg("msrv-prep")
                .arg("--package")
                .arg(package)
                .arg("-vvvv")
                .assert();

            assert.success();

            let unchanged = ["", "member_a", "member_b", "member_c"]
                .iter()
                .copied()
                .filter(|&dir| dir != package_dir)
                .collect::<Vec<_>>();

            validate_workspace_result(&temp, [package_dir], unchanged);
        }

        #[test_log::test]
        fn root_package() {
            test_with_package("test-workspace", "");
        }

        #[test_log::test]
        fn member_a() {
            test_with_package("test-workspace-member-a", "member_a");
        }

        #[test_log::test]
        fn member_b() {
            test_with_package("test-workspace-member-b", "member_b");
        }

        #[test_log::test]
        fn member_c() {
            test_with_package("test-workspace-member-c", "member_c");
        }
    }

    mod excluded_packages_tests {
        use super::*;

        fn test_without_package(package: &str, package_dir: &str) {
            let temp = fork_project("workspace");

            let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
            let assert = cmd
                .current_dir(temp.path())
                .arg("msrv-prep")
                .arg("--workspace")
                .arg("--exclude")
                .arg(package)
                .arg("-vvvv")
                .assert();

            assert.success();

            let changed = ["", "member_a", "member_b", "member_c"]
                .iter()
                .copied()
                .filter(|&dir| dir != package_dir)
                .collect::<Vec<_>>();

            validate_workspace_result(&temp, changed, [package_dir]);
        }

        #[test_log::test]
        fn without_root_package() {
            test_without_package("test-workspace", "");
        }

        #[test_log::test]
        fn without_member_a() {
            test_without_package("test-workspace-member-a", "member_a");
        }

        #[test_log::test]
        fn without_member_b() {
            test_without_package("test-workspace-member-b", "member_b");
        }

        #[test_log::test]
        fn without_member_c() {
            test_without_package("test-workspace-member-c", "member_c");
        }
    }
}

mod no_changes {
    use assert_cmd::Command;
    use assert_fs::assert::PathAssert;
    use assert_fs::fixture::PathChild;
    use predicates::path::missing;

    use super::*;

    #[test_log::test]
    fn no_op() {
        let temp = fork_project("no_changes");

        let mut cmd = Command::cargo_bin(MSRV_PREP_BIN_NAME).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("-vvvv")
            .assert();

        assert.success();

        assert!(toml_files_equal(
            temp.child("expected.toml").path(),
            temp.child("Cargo.toml").path()
        ));
        temp.child("Cargo.toml.msrv-prep.bak").assert(missing());
    }
}
