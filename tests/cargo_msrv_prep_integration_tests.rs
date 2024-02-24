use std::path::PathBuf;

use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;
use predicates::path::eq_file;
use predicates::Predicate;
use predicates::str::PredicateStrExt;

fn fork_project(project_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    let project_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "tests", project_name]
        .iter()
        .collect();
    temp.copy_from(project_path, &["*.rs", "*.toml"]).unwrap();

    temp
}

fn eq_text_file<P>(path: P) -> impl Predicate<str>
where
    P: Into<PathBuf>,
{
    eq_file(path).utf8().unwrap().normalize()
}

mod simple_project_tests {
    use assert_cmd::{crate_name, Command};
    use assert_fs::assert::PathAssert;
    use assert_fs::fixture::PathChild;

    use super::*;

    #[test]
    fn all() {
        let temp = fork_project("simple_project").into_persistent();

        let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
        let assert = cmd.current_dir(temp.path()).arg("msrv-prep").assert();

        assert.success();

        temp.child("Cargo.toml")
            .assert(eq_text_file(temp.child("expected").child("all.toml").path()));
    }

    #[test]
    fn no_remove_rust_version() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--no-remove-rust-version")
            .assert();

        assert.success();

        temp.child("Cargo.toml").assert(eq_text_file(
            temp.child("expected")
                .child("no-remove-rust-version.toml")
                .path(),
        ));
    }

    #[test]
    fn no_merge_pinned_dependencies() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--no-merge-pinned-dependencies")
            .assert();

        assert.success();

        temp.child("Cargo.toml").assert(eq_text_file(
            temp.child("expected")
                .child("no-merge-pinned-dependencies.toml")
                .path(),
        ));
    }
}

mod workspace_tests {
    use assert_cmd::{crate_name, Command};
    use assert_fs::assert::PathAssert;
    use assert_fs::fixture::PathChild;
    use predicates::path::missing;

    use super::*;

    fn validate_workspace_result<'a, 'b, C, U>(temp: &TempDir, changed: C, unchanged: U)
    where
        C: IntoIterator<Item = &'a str>,
        U: IntoIterator<Item = &'b str>,
    {
        let project_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "tests", "workspace"]
            .iter()
            .collect();

        for package in changed {
            temp.child(package)
                .child("Cargo.toml")
                .assert(eq_text_file(temp.child(package).child("expected.toml").path()));
            temp.child(package)
                .child("Cargo.toml.msrv-prep.bak")
                .assert(eq_text_file(project_path.join(package).join("Cargo.toml")));
        }

        for package in unchanged {
            temp.child(package)
                .child("Cargo.toml")
                .assert(eq_text_file(project_path.join(package).join("Cargo.toml")));
            temp.child(package)
                .child("Cargo.toml.msrv-prep.bak")
                .assert(missing());
        }
    }

    #[test]
    fn all() {
        let temp = fork_project("workspace");

        let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
        let assert = cmd
            .current_dir(temp.path())
            .arg("msrv-prep")
            .arg("--workspace")
            .assert();

        assert.success();

        validate_workspace_result(&temp, ["", "member_a", "member_b", "member_c"], []);
    }

    mod specific_packages_tests {
        use super::*;

        fn test_with_package(package: &str, package_dir: &str) {
            let temp = fork_project("workspace");

            let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
            let assert = cmd
                .current_dir(temp.path())
                .arg("msrv-prep")
                .arg("--package")
                .arg(package)
                .assert();

            assert.success();

            let unchanged = ["", "member_a", "member_b", "member_c"]
                .iter()
                .copied()
                .filter(|&dir| dir != package_dir)
                .collect::<Vec<_>>();

            validate_workspace_result(&temp, [package_dir], unchanged);
        }

        #[test]
        fn root_package() {
            test_with_package("test-workspace", "");
        }

        #[test]
        fn member_a() {
            test_with_package("test-workspace-member-a", "member_a");
        }

        #[test]
        fn member_b() {
            test_with_package("test-workspace-member-b", "member_b");
        }

        #[test]
        fn member_c() {
            test_with_package("test-workspace-member-c", "member_c");
        }
    }

    mod excluded_packages_tests {
        use super::*;

        fn test_without_package(package: &str, package_dir: &str) {
            let temp = fork_project("workspace");

            let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
            let assert = cmd
                .current_dir(temp.path())
                .arg("msrv-prep")
                .arg("--workspace")
                .arg("--exclude")
                .arg(package)
                .assert();

            assert.success();

            let changed = ["", "member_a", "member_b", "member_c"]
                .iter()
                .copied()
                .filter(|&dir| dir != package_dir)
                .collect::<Vec<_>>();

            validate_workspace_result(&temp, changed, [package_dir]);
        }

        #[test]
        fn without_root_package() {
            test_without_package("test-workspace", "");
        }

        #[test]
        fn without_member_a() {
            test_without_package("test-workspace-member-a", "member_a");
        }

        #[test]
        fn without_member_b() {
            test_without_package("test-workspace-member-b", "member_b");
        }

        #[test]
        fn without_member_c() {
            test_without_package("test-workspace-member-c", "member_c");
        }
    }
}
