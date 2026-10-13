#!/usr/bin/env bash
set -euo pipefail

benchmark_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
results_dir="$benchmark_dir/results"
runs=5

mkdir -p "$results_dir"

manifests=(
    "$benchmark_dir/diesel/Cargo.toml"
    "$benchmark_dir/sqlx/Cargo.toml"
    "$benchmark_dir/placeholder-query/Cargo.toml"
)
labels=(
    "diesel-pagila"
    "sqlx-pagila"
    "placeholder-query-pagila"
)

rustflags="-C link-arg=-fuse-ld=mold"
crate_target_dirs=(
    "$benchmark_dir/diesel/target/crate"
    "$benchmark_dir/sqlx/target/crate"
    "$benchmark_dir/placeholder-query/target/crate"
)

for index in "${!manifests[@]}"; do
    env CARGO_INCREMENTAL=0 RUSTC_WRAPPER= RUSTFLAGS="$rustflags" cargo build --locked \
        --manifest-path "${manifests[$index]}" \
        --target-dir "${crate_target_dirs[$index]}"
    env CARGO_INCREMENTAL=0 RUSTC_WRAPPER= RUSTFLAGS="$rustflags" cargo build --locked --release \
        --manifest-path "${manifests[$index]}" \
        --target-dir "${crate_target_dirs[$index]}"
done

prepare_commands=()
dev_commands=()
release_crate_commands=()
for index in "${!manifests[@]}"; do
    manifest="${manifests[$index]}"
    target_dir="${crate_target_dirs[$index]}"
    prepare_commands+=("touch '${manifest%/Cargo.toml}/src/lib.rs'")
    dev_commands+=("env CARGO_INCREMENTAL=0 RUSTC_WRAPPER= RUSTFLAGS='$rustflags' cargo build --locked --manifest-path '$manifest' --target-dir '$target_dir'")
    release_crate_commands+=("env CARGO_INCREMENTAL=0 RUSTC_WRAPPER= RUSTFLAGS='$rustflags' cargo build --locked --release --manifest-path '$manifest' --target-dir '$target_dir'")
done
prepare_command="${prepare_commands[0]}"
for prepare_step in "${prepare_commands[@]:1}"; do
    prepare_command+=" && $prepare_step"
done

hyperfine \
    --warmup 1 \
    --runs "$runs" \
    --prepare "$prepare_command" \
    --command-name "${labels[0]}" "${dev_commands[0]}" \
    --command-name "${labels[1]}" "${dev_commands[1]}" \
    --command-name "${labels[2]}" "${dev_commands[2]}" \
    --export-json "$results_dir/pagila-development-crate.json"

hyperfine \
    --warmup 1 \
    --runs "$runs" \
    --prepare "$prepare_command" \
    --command-name "${labels[0]}" "${release_crate_commands[0]}" \
    --command-name "${labels[1]}" "${release_crate_commands[1]}" \
    --command-name "${labels[2]}" "${release_crate_commands[2]}" \
    --export-json "$results_dir/pagila-release-crate.json"

printf '\nMeasurement\tLibrary\tTrimmed mean (s)\n'
for measurement in pagila-development-crate pagila-release-crate; do
    jq -r \
        --arg measurement "$measurement" \
        '.results[] | [
            $measurement,
            .command,
            (.times | sort | .[1:-1] | add / length)
        ] | @tsv' \
        "$results_dir/$measurement.json"
done
