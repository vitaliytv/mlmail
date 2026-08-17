#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../src-tauri"
WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
PACKAGE_PATH="${WORKSPACE_ROOT}/target/draft-helper.n-plugin"
N_PLUGIN_BIN="${N_PLUGIN_BIN:-n-plugin}"

RUSTUP_TOOLCHAIN=stable cargo build --offline \
  --package draft-helper \
  --target wasm32-wasip2

"${N_PLUGIN_BIN}" build \
  --manifest "${WORKSPACE_ROOT}/app/src-tauri/plugins/draft-helper/.n-plugin.toml" \
  --component "${WORKSPACE_ROOT}/target/wasm32-wasip2/debug/draft_helper.wasm" \
  --output "${PACKAGE_PATH}"

MLMAIL_DRAFT_HELPER_COMPONENT="${PACKAGE_PATH}" \
  RUSTUP_TOOLCHAIN=stable cargo test --offline \
  --package mlmail \
  plugins::tests::installs_packaged_component_into_the_activation_registry \
  -- --ignored

MLMAIL_DRAFT_HELPER_COMPONENT="${PACKAGE_PATH}" \
  RUSTUP_TOOLCHAIN=stable cargo test --offline \
  --package mlmail \
  plugin_draft_helper::tests::invokes_packaged_draft_helper_through_typed_gmail_drafts \
  -- --ignored
