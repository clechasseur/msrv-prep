# Rust project template

[![CI](https://github.com/clechasseur/rust-template/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/clechasseur/rust-template/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/clechasseur/rust-template/branch/main/graph/badge.svg?token=qSFdAkbb8U)](https://codecov.io/gh/clechasseur/rust-template) [![Security audit](https://github.com/clechasseur/rust-template/actions/workflows/audit-check.yml/badge.svg?branch=main)](https://github.com/clechasseur/rust-template/actions/workflows/audit-check.yml) [![crates.io](https://img.shields.io/crates/v/rust-template-clp.svg)](https://crates.io/crates/rust-template-clp) [![downloads](https://img.shields.io/crates/d/rust-template-clp.svg)](https://crates.io/crates/rust-template-clp) [![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/rust-template-clp) [![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

This is a simple template repository for Rust projects that includes some default workflows and configuration files.

## Usage

1. Create a new repository using this template (_Note_: do not include all branches, unless you want to end up with the test branch)
2. Clone your new repository
3. Run `cargo init` to create a Rust project at the repository root<br/>
   OR<br/>
   Run `cargo new <project>` from the repository root to create a new Rust project, then create a root `Cargo.toml` to setup a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
4. Adjust workflows as required
   * In particular, in order to have code coverage work, setup your project on [codecov.io](https://about.codecov.io/) and create a `CODECOV_TOKEN` secret for your repository's actions
5. Adjust/add/remove status badges in this README
6. Adjust links in [CONTRIBUTING.md](./CONTRIBUTING.md), [DEVELOPMENT.md](./DEVELOPMENT.md), [SECURITY.md](./SECURITY.md) and [PULL_REQUEST_TEMPLATE.md](./.github/PULL_REQUEST_TEMPLATE.md)
7. ???
8. Profit!
