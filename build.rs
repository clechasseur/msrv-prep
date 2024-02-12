use rustc_version::version_meta;
use rustc_version::Channel::Nightly;

fn main() {
    if version_meta().unwrap().channel <= Nightly {
        println!("cargo:rustc-cfg=nightly_rustc");
    }
}
