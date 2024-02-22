use std::path::PathBuf;

use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;

fn fork_project(project_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    let project_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "tests", project_name]
        .iter()
        .collect();
    temp.copy_from(project_path, &["*.rs", "*.toml"]).unwrap();

    temp
}

mod simple_project_tests {
    use assert_cmd::{crate_name, Command};
    use assert_fs::assert::PathAssert;
    use assert_fs::fixture::PathChild;
    use predicates::path::eq_file;

    use super::*;

    #[test]
    fn all() {
        let temp = fork_project("simple_project");

        let mut cmd = Command::cargo_bin(crate_name!()).unwrap();
        let assert = cmd.current_dir(temp.path()).arg("msrv-prep").assert();

        assert.success();

        temp.child("Cargo.toml")
            .assert(eq_file(temp.child("expected").child("all.toml").path()));
    }
}
