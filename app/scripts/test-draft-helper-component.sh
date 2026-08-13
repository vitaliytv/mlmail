#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../src-tauri"

RUSTUP_TOOLCHAIN=stable cargo build --offline \
  --package draft-helper \
  --target wasm32-wasip2

MLMAIL_DRAFT_HELPER_COMPONENT="${PWD}/target/wasm32-wasip2/debug/draft_helper.wasm" \
  RUSTUP_TOOLCHAIN=stable cargo test --offline \
  --package mlmail \
  plugin_draft_helper::tests::invokes_packaged_draft_helper_through_typed_gmail_drafts \
  -- --ignored
