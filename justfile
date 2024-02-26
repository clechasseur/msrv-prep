set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

toolchain := ''
trimmed_toolchain := trim(toolchain)

cargo := if trimmed_toolchain != "" {
    "cargo +" + trimmed_toolchain
} else {
    "cargo"
}

all_features := "true"
all_features_flag := if all_features == "true" { "--all-features" } else { "" }

all_targets := "true"
all_targets_flag := if all_targets == "true" { "--all-targets" } else { "" }

message_format := ""
message_format_flag := if message_format != "" { "--message-format " + message_format } else { "" }

default:
    @just --list

tidy: clippy fmt

clippy:
    {{cargo}} clippy --workspace {{all_targets_flag}} {{all_features_flag}} -- -D warnings

fmt:
    cargo +nightly fmt --all

check *extra_args:
    {{cargo}} check --workspace {{all_targets_flag}} {{all_features_flag}} {{message_format_flag}} {{extra_args}}

build *extra_args:
    {{cargo}} build --workspace {{all_targets_flag}} {{all_features_flag}} {{message_format_flag}} {{extra_args}}

test *extra_args:
    {{cargo}} test --workspace {{all_features_flag}} {{message_format_flag}} {{extra_args}}

update *extra_args:
    {{cargo}} update {{extra_args}}

@tarpaulin *extra_args:
    # Note: there seems to be an issue in `cargo-tarpaulin` when using Rust 1.75.0 or later - it reports some missing line coverage.
    # I've entered an issue: https://github.com/xd009642/tarpaulin/issues/1438
    # In the meantime, let's pin the Rust version used for code coverage to 1.74.1 until we know what's happening.
    @cargo +1.74.1 tarpaulin --target-dir target-tarpaulin {{extra_args}}
    {{ if env('CI', '') == '' { `just _open-tarpaulin` } else { ` ` } }}

[unix]
@_open-tarpaulin:
    open tarpaulin-report.html

[windows]
@_open-tarpaulin:
    ./tarpaulin-report.html

doc $RUSTDOCFLAGS="-D warnings":
    {{cargo}} doc {{ if env('CI', '') != '' { '--no-deps' } else { '--open' } }} --workspace {{all_features_flag}} {{message_format_flag}}

doc-coverage $RUSTDOCFLAGS="-Z unstable-options --show-coverage":
    cargo +nightly doc --no-deps --workspace {{all_features_flag}} {{message_format_flag}}

backup-manifest lockfile_bak="Cargo.lock.bak":
    {{ if path_exists(lockfile_bak) == "true" { "rm " + lockfile_bak } else { "" } }}
    {{ if path_exists("Cargo.lock") == "true" { "mv Cargo.lock " + lockfile_bak } else { "" } }}

restore-manifest lockfile_bak="Cargo.lock.bak":
    {{ if path_exists("Cargo.lock") == "true" { "rm Cargo.lock" } else { "" } }}
    {{ if path_exists(lockfile_bak) == "true" { "mv " + lockfile_bak + " Cargo.lock" } else { "" } }}

minimize:
    cargo hack --remove-dev-deps --workspace
    cargo +nightly update -Z minimal-versions

check-minimal-only:
    {{cargo}} minimal-versions check --workspace --lib --bins {{all_features_flag}} {{message_format_flag}}

check-minimal: backup-manifest check-minimal-only restore-manifest

msrv-minimal: (backup-manifest "Cargo.lock.bak.msrv") && (restore-manifest "Cargo.lock.bak.msrv")
    cargo msrv -- just check-minimal

msrv:
    cargo msrv -- just check

test-package:
    {{cargo}} publish --dry-run
