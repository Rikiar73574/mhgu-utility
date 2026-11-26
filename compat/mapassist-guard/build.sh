#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
if [[ -n "${MHGU_TOOLCHAIN:-}" ]]; then
  export RUSTUP_HOME="$MHGU_TOOLCHAIN/rustup"
  export CARGO_HOME="$MHGU_TOOLCHAIN/cargo"
fi
cargo +nightly build --release --locked --offline
mkdir -p ../../build
rustc +nightly --edition 2024 --test src/roles.rs -o ../../build/mapassist-role-tests
../../build/mapassist-role-tests
python3 ../../tools/elf_to_nro.py target/armv7a-none-eabi/release/libmapassist_role_guard.so ../../build/mapassist_role_guard.nro
# ARM regression harness additionally needs Unicorn and pyelftools:
# python3 tests/arm_roles.py && python3 tests/arm_roles.py --unsupported
