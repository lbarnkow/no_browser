#!/bin/sh

set -e

# most commands taken from: https://blog.rng0.io/how-to-do-code-coverage-in-rust

echo "Install tooling..."
rustup component add llvm-tools-preview
GRCOV_ARCHIVE="grcov-x86_64-unknown-linux-musl.tar.bz2"
wget \
    --output-document "/tmp/${GRCOV_ARCHIVE}" \
    "https://github.com/mozilla/grcov/releases/latest/download/${GRCOV_ARCHIVE}"
tar \
    --extract \
    --directory /tmp/ \
    --file "/tmp/${GRCOV_ARCHIVE}"
rm --force "/tmp/${GRCOV_ARCHIVE}"
/tmp/grcov --version

echo "Clean old coverage data..."
COVERAGE_FOLDER="target/coverage"
rm --recursive --force "${COVERAGE_FOLDER}/"

echo "Compile and run tests to generate coverage data..."
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="${COVERAGE_FOLDER}/no_browser-%p-%m.profraw"
export CARGO_INCREMENTAL=0
cargo test

echo "Tranform coverage data to html reports..."
grcov . \
    --binary-path ./target/debug/deps/ \
    --source-dir src \
    --output-type html \
    --branch \
    --ignore-not-existing \
    --ignore '../*' \
    --ignore "/*" \
    --output-path "${COVERAGE_FOLDER}/html"

echo "Tranform coverage data to lcov format..."
grcov . \
    --binary-path ./target/debug/deps/ \
    --source-dir src \
    --output-type lcov \
    --branch \
    --ignore-not-existing \
    --ignore '../*' \
    --ignore "/*" \
    --output-path "${COVERAGE_FOLDER}/lcov.info"

echo "Coverage done!"
