#!/usr/bin/env bash
# Build the official Voltrus binary: open core (core/ submodule) + Enterprise add-on (enterprise/).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
git -C "$ROOT" submodule update --init core

# Web client: core client plus the enterprise module. Writes core/static, which the release
# builds below embed into the executable (so this step comes first).
(
  cd "$ROOT/core/client"
  npm ci
  VOLTRUS_ENTERPRISE_CLIENT="$ROOT/enterprise/client/src/index.ts" npm run build
)

# Server: the enterprise crate is the `voltrus` binary (it calls voltrus::run with the add-on).
(cd "$ROOT/enterprise" && cargo build --release --locked --bin voltrus)
# Core-only helper binaries (simulators).
cargo build --release --locked --manifest-path "$ROOT/core/Cargo.toml" --bin modbus-sim
echo "official binary: $ROOT/enterprise/target/release/voltrus (web UI embedded)"
