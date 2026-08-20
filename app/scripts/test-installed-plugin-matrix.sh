#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel)"
SRC_TAURI="${WORKSPACE_ROOT}/app/src-tauri"
N_PLUGIN_BIN="${N_PLUGIN_BIN:-n-plugin}"
PLUGIN_E2E_REGISTRY="${MLMAIL_PLUGIN_E2E_REGISTRY:-git.7n.ai}"
PLUGIN_E2E_RELEASE="${MLMAIL_PLUGIN_E2E_RELEASE:-vitaliytv:mlmail-e2e-root@0.1.0}"
PLUGIN_E2E_DIGEST="${MLMAIL_PLUGIN_E2E_DIGEST:-sha256:5272fe568bbe2f6a816f4ea64c32e61b3d6c2af6c7d53a11b136ad59798c70a7}"
ARTIFACT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/mlmail-plugin-matrix.XXXXXX")"

cleanup() {
  case "${ARTIFACT_DIR}" in
    "${TMPDIR:-/tmp}"/mlmail-plugin-matrix.*) rm -rf -- "${ARTIFACT_DIR}" ;;
    *) echo "Refusing to remove unexpected matrix directory: ${ARTIFACT_DIR}" >&2 ;;
  esac
}
trap cleanup EXIT

fail() {
  echo "Plugin matrix: $*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command '$1' is unavailable"
}

require_n_plugin() {
  if [[ "${N_PLUGIN_BIN}" == */* ]]; then
    [[ -x "${N_PLUGIN_BIN}" ]] || fail "N_PLUGIN_BIN must point to an executable n-plugin binary"
  else
    command -v "${N_PLUGIN_BIN}" >/dev/null 2>&1 ||
      fail "n-plugin is unavailable; set N_PLUGIN_BIN to the CLI executable"
  fi
}

inspect_release() {
  local component_path="$1"
  local expected_package="$2"
  local expected_version="$3"
  local expected_digest="${4:-}"
  local inspection

  inspection="$("${N_PLUGIN_BIN}" inspect "${component_path}")"
  printf '%s\n' "${inspection}"
  printf '%s\n' "${inspection}" | jq -e \
    --arg package "${expected_package}" \
    --arg version "${expected_version}" \
    --arg digest "${expected_digest}" '
      .release.package == $package
      and .release.version == $version
      and (.release.digest | test("^sha256:[0-9a-f]{64}$"))
      and ($digest == "" or .release.digest == $digest)
    ' >/dev/null
}

package_component() {
  local manifest_path="$1"
  local component_path="$2"
  local output_path="$3"

  "${N_PLUGIN_BIN}" build \
    --manifest "${manifest_path}" \
    --component "${component_path}" \
    --output "${output_path}"
}

draft_manifest() {
  local publisher="$1"
  local package="$2"
  local output_path="$3"

  sed \
    -e "s/^publisher_id = \"nitra\"$/publisher_id = \"${publisher}\"/" \
    -e "s/^package = \"draft-helper\"$/package = \"${package}\"/" \
    "${SRC_TAURI}/plugins/draft-helper/.n-plugin.toml" >"${output_path}"
}

run_rust_test() {
  local filter="$1"
  RUSTUP_TOOLCHAIN=stable cargo test --offline \
    --manifest-path "${SRC_TAURI}/Cargo.toml" \
    --package mlmail \
    --lib \
    "${filter}"
}

run_ignored_component_test() {
  local environment_name="$1"
  local component_path="$2"
  local filter="$3"

  env "${environment_name}=${component_path}" \
    RUSTUP_TOOLCHAIN=stable cargo test --offline \
    --manifest-path "${SRC_TAURI}/Cargo.toml" \
    --package mlmail \
    --lib \
    "${filter}" \
    -- --ignored
}

run_oci_smoke() {
  local release="${PLUGIN_E2E_RELEASE}"
  local expected_digest="${PLUGIN_E2E_DIGEST}"
  local package
  local version
  local fetched="${ARTIFACT_DIR}/oci-root.n-plugin"
  local lock_file="${ARTIFACT_DIR}/oci-root.n-plugin.lock"
  local cache="${ARTIFACT_DIR}/oci-cache"

  [[ "${release}" == *:*@* ]] ||
    fail "MLMAIL_PLUGIN_E2E_RELEASE must use namespace:package@version"
  [[ "${expected_digest}" =~ ^sha256:[0-9a-f]{64}$ ]] ||
    fail "MLMAIL_PLUGIN_E2E_DIGEST must pin the published fixture digest"
  package="${release%@*}"
  version="${release##*@}"

  echo "Plugin matrix: fetching exact public OCI fixture ${release}"
  "${N_PLUGIN_BIN}" fetch "${release}" \
    --registry "${PLUGIN_E2E_REGISTRY}" \
    --output "${fetched}"
  inspect_release "${fetched}" "${package}" "${version}" "${expected_digest}"

  "${N_PLUGIN_BIN}" lock "${fetched}" \
    --registry "${PLUGIN_E2E_REGISTRY}" \
    --lock-file "${lock_file}" \
    --cache "${cache}"
  "${N_PLUGIN_BIN}" lock "${fetched}" \
    --registry "${PLUGIN_E2E_REGISTRY}" \
    --lock-file "${lock_file}" \
    --cache "${cache}" \
    --offline
}

require_command cargo
require_command bun
require_command git
require_command jq
require_command rustup
require_command sed
require_n_plugin

if ! RUSTUP_TOOLCHAIN=stable rustup target list --installed | grep -Fxq wasm32-wasip2; then
  fail "Rust target wasm32-wasip2 is missing; run 'rustup target add --toolchain stable wasm32-wasip2'"
fi

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0

echo "Plugin matrix: building independent Rust Components"
RUSTUP_TOOLCHAIN=stable cargo build --offline \
  --manifest-path "${SRC_TAURI}/Cargo.toml" \
  --package draft-helper \
  --package booking-finder \
  --target wasm32-wasip2

DRAFT_WASM="${WORKSPACE_ROOT}/target/wasm32-wasip2/debug/draft_helper.wasm"
BOOKING_WASM="${WORKSPACE_ROOT}/target/wasm32-wasip2/debug/booking_finder.wasm"
ALPHA_MANIFEST="${ARTIFACT_DIR}/alpha-draft-helper.toml"
ZETA_MANIFEST="${ARTIFACT_DIR}/zeta-draft-helper.toml"
ALPHA_COMPONENT="${ARTIFACT_DIR}/alpha-draft-helper.n-plugin"
ZETA_COMPONENT="${ARTIFACT_DIR}/zeta-draft-helper.n-plugin"
BOOKING_COMPONENT="${ARTIFACT_DIR}/booking-finder.n-plugin"

draft_manifest alpha draft-helper "${ALPHA_MANIFEST}"
draft_manifest zeta draft-helper "${ZETA_MANIFEST}"
package_component "${ALPHA_MANIFEST}" "${DRAFT_WASM}" "${ALPHA_COMPONENT}"
package_component "${ZETA_MANIFEST}" "${DRAFT_WASM}" "${ZETA_COMPONENT}"
package_component \
  "${SRC_TAURI}/plugins/booking-finder/.n-plugin.toml" \
  "${BOOKING_WASM}" \
  "${BOOKING_COMPONENT}"

inspect_release "${ALPHA_COMPONENT}" "alpha:draft-helper" "0.1.0"
inspect_release "${ZETA_COMPONENT}" "zeta:draft-helper" "0.1.0"
inspect_release "${BOOKING_COMPONENT}" "nitra:booking-finder" "0.1.0"

echo "Plugin matrix: invoking both Draft publishers and Booking Finder through typed adapters"
run_ignored_component_test \
  MLMAIL_DRAFT_HELPER_COMPONENT \
  "${ALPHA_COMPONENT}" \
  plugin_draft_helper::tests::invokes_packaged_draft_helper_through_typed_gmail_drafts
run_ignored_component_test \
  MLMAIL_DRAFT_HELPER_COMPONENT \
  "${ZETA_COMPONENT}" \
  plugin_draft_helper::tests::invokes_packaged_draft_helper_through_typed_gmail_drafts
run_ignored_component_test \
  MLMAIL_BOOKING_FINDER_COMPONENT \
  "${BOOKING_COMPONENT}" \
  plugin_booking_finder::tests::invokes_packaged_booking_finder_through_typed_gmail_search

echo "Plugin matrix: checking preflight, consent, exact dispatch, graph cache and offline restart"
run_rust_test plugin_install::tests::previews_draft_helper_and_booking_finder_through_one_path
run_rust_test plugin_install::tests::returns_incompatible_preview_for_unknown
run_rust_test plugin_install::tests::binds_dependency_consent_to_one_exact_generated_edge
run_rust_test plugins::tests::confirms_an_exact_dependency_graph_and_rejects_tampered_offline_cache
run_rust_test plugins::tests::dispatches_two_draft_helpers_by_exact_release_and_fails_closed
run_rust_test plugin_context::tests::replays_offline_context_and_repairs_an_interrupted_process

echo "Plugin matrix: checking exact Vue action and lifecycle command payloads"
bun run --cwd "${WORKSPACE_ROOT}/app" test -- src/components/tests/PluginManagerPanel.test.mjs

if [[ "${MLMAIL_PLUGIN_E2E_ONLINE:-0}" == "1" ]]; then
  run_oci_smoke
else
  echo "Plugin matrix: OCI smoke skipped; set MLMAIL_PLUGIN_E2E_ONLINE=1 with an exact public fixture"
fi

echo "Plugin matrix: all requested checks passed"
