# `cargo-msrv-prep`

[![CI](https://github.com/clechasseur/msrv-prep/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/msrv-prep/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/msrv-prep/branch/main/graph/badge.svg?token=y4eecxkGcV)](https://codecov.io/gh/clechasseur/msrv-prep) [![Security audit](https://github.com/clechasseur/msrv-prep/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/msrv-prep/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/cargo-msrv-prep.svg)](https://crates.io/crates/cargo-msrv-prep) [![downloads](https://img.shields.io/crates/d/cargo-msrv-prep.svg)](https://crates.io/crates/cargo-msrv-prep) [![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/cargo-msrv-prep) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

A Cargo subcommand useful to prepare for determining/verifying a Rust crate's [MSRV](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field) (_Minimum Supported Rust Version_).

## Installing

Via `cargo`:

```sh
cargo install cargo-msrv-prep --locked
```

Via [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall cargo-msrv-prep
```

You can also install it manually by downloading the appropriate executable from the [project's Releases page](https://github.com/clechasseur/msrv-prep/releases).

### Dependencies

This tool is meant as a companion to the following tools (which are much cooler):

* [`cargo-msrv`](https://github.com/foresterre/cargo-msrv): Cargo command to determine a crate's MSRV
* [`cargo-hack`](https://github.com/taiki-e/cargo-hack): Cargo command providing tools for testing and CI
* [`cargo-minimal-versions`](https://github.com/taiki-e/cargo-minimal-versions): Cargo command to fetch minimal dependencies (uses `cargo-hack` above)

You can visit these tools' GitHub pages to learn how to install them if you do not have them installed already.

## Usage

Determining a Rust crate's Minimum Supported Rust Version is not so easy.
The [`cargo-msrv`](https://github.com/foresterre/cargo-msrv) tool is designed for this and allows running a check command (by default `cargo check`) using multiple Rust versions until the MSRV is found.
However, there are a couple of issues with this:

* If your crate has a `rust-version` field, it won't allow `cargo-msrv` to "see" a working lower MSRV.
* In order to determine the _true_ MSRV, you need to build with the _minimum_ supported versions of all dependencies (transitively), but this is not the default behavior of Cargo.

The second issue can be partially circumvented by using a nifty tool called [`cargo-minimal-versions`](https://github.com/taiki-e/cargo-minimal-versions).
It allows running Cargo commands using the unstable [`-Z minimal-versions`](https://github.com/rust-lang/cargo/issues/5657) option to fetch minimum versions of dependencies.
By using this, we can run this to determine our crate's true MSRV:

```sh
cargo msrv -- cargo minimal-versions check --workspace --lib --bins --all-features
```

But there remains another issue: some crates specify erroneous minimum versions for their dependencies; the specified minimum versions are actually _too low_ and don't build.
Sometimes, it's because the minimum specified version of the dependency is old, used to build but no longer builds for various reasons (this can happen when that version of the dependency was created before Rust 1.0, for example).
Other times, it can be because the crate's author specified a minimum dependency version that _used_ to work but now no longer works because they are using a feature of the dependency introduced later, but because Cargo always pulls the latest version of dependencies, the crate's author didn't notice.
(This is another good reason why it's a good idea to _check_ that a crate's MSRV remains correct using CI.)

That last issue is unfortunately "unsolvable" - the faulty dependencies cannot be modified since they are published and set in stone.
The only solution is to "pin" some of the faulty dependencies to more recent versions through your workspace's `Cargo.toml` file.

This is where `cargo-msrv-prep` comes in. It loads a manifest (`Cargo.toml` file) and does two things:

* If the manifest has a `rust-version` field (in the `package` table), it is removed
* If a file named `msrv-pins.toml` exists next to the manifest, any dependencies specified in that file are merged with those in the manifest

For example, if your project had [this `Cargo.toml` file](./resources/tests/cargo-msrv-prep/simple_project/Cargo.toml) and [this `msrv-pins.toml` file](./resources/tests/cargo-msrv-prep/simple_project/msrv-pins.toml), running `cargo msrv-prep` would produce [this output](./resources/tests/cargo-msrv-prep/simple_project/expected/all.toml) (replacing the `Cargo.toml` file).

(It's possible to override the name of the `msrv-pins.toml` file, change the backup file suffix, etc. Run `cargo msrv-prep --help` for all options.)

Running `cargo-msrv-prep` will back up all modified manifests. Another Cargo command, `cargo-msrv-unprep`, is provided to reverse the process.

Consequently, you can use this tool to determine the true MSRV of your crate without needing to hack the `Cargo.toml` file by hand by running:

```sh
cargo msrv-prep --workspace
cargo msrv -- cargo minimal-versions check --workspace --lib --bins --all-features
cargo msrv-unprep --workspace
```

In order to _validate_ that the MSRV specified in your crate's manifest is correct, you can use `cargo-msrv-prep` like this:

```sh
cargo msrv-prep --workspace
cargo minimal-versions check --workspace --lib --bins --all-features
```

Here's an example of a GitHub workflow to perform this validation in your CI.
This workflow uses [`taiki-e/install-action`](https://github.com/taiki-e/install-action) to install the required tools.

```yaml
name: MSRV check

on: [push]

jobs:
  msrv-check:
    name: MSRV check for Rust ${{ matrix.toolchain }} on ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        toolchain: [ 1.74.1 ]          # Set this to the expected MSRV of your crate
        os: [ ubuntu, macos, windows ] # It's probably a good idea to run this check on all supported OSes
    runs-on: ${{ matrix.os }}-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust nightly toolchain # Required for `cargo-minimal-versions` to work
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: nightly
          cache: false

      - name: Install Rust minimum supported toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: ${{ matrix.toolchain }}
          cache: false
  
      # If you want to use the `rust-cache` action, it's probably a good idea to make your cache key
      # conditional on the `msrv-pins.toml` file(s) since they will affect the resulting build
      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          key: msrv-pins-files-${{ hashFiles('**/msrv-pins.toml') }}

      - name: Install required tools
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-hack,cargo-minimal-versions,cargo-msrv-prep

      - name: Run checks using cargo-minimal-versions
        run: |
          cargo msrv-prep --workspace
          cargo minimal-versions check --workspace --lib --bins --all-features
```

## MSRV of `cargo-msrv-prep`

The MSRV of the `cargo-msrv-prep` tool is Rust **1.74.1**.
This is only important when installing from source (e.g. with `cargo install`) however, since the executable will then work with older Rust versions.
