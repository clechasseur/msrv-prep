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

tarpaulin *extra_args:
    {{cargo}} tarpaulin --target-dir target-tarpaulin {{extra_args}}
    {{ if env('CI', '') == '' { `open tarpaulin-report.html` } else { ` ` } }}

doc $RUSTDOCFLAGS="-D warnings":
    {{cargo}} doc {{ if env('CI', '') != '' { '--no-deps' } else { '--open' } }} --workspace {{all_features_flag}} {{message_format_flag}}

doc-coverage $RUSTDOCFLAGS="-Z unstable-options --show-coverage":
    cargo +nightly doc --no-deps --workspace {{all_features_flag}} {{message_format_flag}}

backup-manifest manifest_bak="Cargo.toml.bak" lockfile_bak="Cargo.lock.bak":
    {{ if path_exists(manifest_bak) == "true" { "rm " + manifest_bak } else { "" } }}
    {{ if path_exists(lockfile_bak) == "true" { "rm " + lockfile_bak } else { "" } }}
    {{ if path_exists("Cargo.toml") == "true" { "mv Cargo.toml " + manifest_bak } else { "" } }}
    {{ if path_exists("Cargo.lock") == "true" { "mv Cargo.lock " + lockfile_bak } else { "" } }}

restore-manifest manifest_bak="Cargo.toml.bak" lockfile_bak="Cargo.lock.bak":
    {{ if path_exists("Cargo.toml") == "true" { "rm Cargo.toml" } else { "" } }}
    {{ if path_exists("Cargo.lock") == "true" { "rm Cargo.lock" } else { "" } }}
    {{ if path_exists(manifest_bak) == "true" { "mv " + manifest_bak + " Cargo.toml" } else { "" } }}
    {{ if path_exists(lockfile_bak) == "true" { "mv " + lockfile_bak + " Cargo.lock" } else { "" } }}

apply-msrv:
    cp Cargo.toml.msrv Cargo.toml

save-msrv:
    cp Cargo.toml Cargo.toml.msrv

minimize:
    cargo hack --remove-dev-deps --workspace
    cargo +nightly update -Z minimal-versions

check-minimal-only:
    {{cargo}} minimal-versions check --workspace --lib --bins {{all_features_flag}} {{message_format_flag}}

check-minimal: backup-manifest apply-msrv check-minimal-only restore-manifest

msrv: (backup-manifest "Cargo.toml.bak.msrv" "Cargo.lock.bak.msrv") apply-msrv && (restore-manifest "Cargo.toml.bak.msrv" "Cargo.lock.bak.msrv")
    cargo msrv -- just check-minimal

test-package:
    {{cargo}} publish --dry-run
