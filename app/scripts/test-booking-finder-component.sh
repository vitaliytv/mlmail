#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../src-tauri"
WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"

RUSTUP_TOOLCHAIN=stable cargo build --offline \
  --package booking-finder \
  --target wasm32-wasip2

MLMAIL_BOOKING_FINDER_COMPONENT="${WORKSPACE_ROOT}/target/wasm32-wasip2/debug/booking_finder.wasm" \
  RUSTUP_TOOLCHAIN=stable cargo test --offline \
  --package mlmail \
  plugin_booking_finder::tests::invokes_packaged_booking_finder_through_typed_gmail_search \
  -- --ignored
