---
type: Module
title: manager.rs
resource: app/src-tauri/mlmail-plugin-host/src/manager.rs
docgen:
  crc: 800290cc
---

## Огляд

Plugin Manager lifecycle: preview/install `.n-plugin` з consent-diff і TOFU, signed sample, disable, uninstall purge, draft через встановлений Wasm.

## Публічний API

- `preview_install` — verify пакета; ключ з companion `.pub` або trust store
- `install_with_consent` — install після згоди; `public_key_hex` або companion `.pub`
- `install_sample_draft_helper` — compile WAT→wasm, sign, pack `sample.n-plugin`, install signed
- `create_draft_from_installed` — `ensure_invocable` + load `component.wasm` + MockMailHost draft
- `set_disabled` / `ensure_invocable` / `uninstall_purge` / `list_managed` / `diff_capabilities`
