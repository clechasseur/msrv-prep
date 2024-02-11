use std::env;

// use rustc_version::version_meta;
// use rustc_version::Channel::Nightly;

fn main() {
    println!("cargo:rerun-if-env-changed=CI");

    // Uncomment this to be notified when we're building on Nightly toolset
    // if version_meta().unwrap().channel <= Nightly {
    //     println!("cargo:rustc-cfg=nightly_rustc");
    // }

    // On CI, do not run integration tests.
    if env::var("CI").is_ok() {
        println!("cargo:rustc-cfg=skip_integration_tests")
    }
}
