#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../src-tauri"
WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
PACKAGE_PATH="${WORKSPACE_ROOT}/target/booking-finder.n-plugin"
N_PLUGIN_BIN="${N_PLUGIN_BIN:-n-plugin}"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to validate the packaged plugin identity" >&2
  exit 1
fi

if [[ "${N_PLUGIN_BIN}" == */* ]]; then
  if [[ ! -x "${N_PLUGIN_BIN}" ]]; then
    echo "N_PLUGIN_BIN must point to an executable n-plugin binary" >&2
    exit 1
  fi
elif ! command -v "${N_PLUGIN_BIN}" >/dev/null 2>&1; then
  echo "n-plugin is unavailable; set N_PLUGIN_BIN to the CLI executable" >&2
  exit 1
fi

RUSTUP_TOOLCHAIN=stable cargo build --offline \
  --package booking-finder \
  --target wasm32-wasip2

"${N_PLUGIN_BIN}" build \
  --manifest "${WORKSPACE_ROOT}/app/src-tauri/plugins/booking-finder/.n-plugin.toml" \
  --component "${WORKSPACE_ROOT}/target/wasm32-wasip2/debug/booking_finder.wasm" \
  --output "${PACKAGE_PATH}"

INSPECTION="$("${N_PLUGIN_BIN}" inspect "${PACKAGE_PATH}")"
printf '%s\n' "${INSPECTION}"
printf '%s\n' "${INSPECTION}" | jq -e '
  .release.package == "nitra:booking-finder"
  and .release.version == "0.1.0"
  and (.release.digest | test("^sha256:[0-9a-f]{64}$"))
' >/dev/null

MLMAIL_BOOKING_FINDER_COMPONENT="${PACKAGE_PATH}" \
  RUSTUP_TOOLCHAIN=stable cargo test --offline \
  --package mlmail \
  --lib \
  plugin_booking_finder::tests::invokes_packaged_booking_finder_through_typed_gmail_search \
  -- --ignored
