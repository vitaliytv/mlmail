---
type: Module
title: plugins.rs
resource: app/src-tauri/src/plugins.rs
docgen:
  crc: 8b65bac0
---

## Огляд

Tauri-команди A2UI sidebar і Plugin Manager: preview/install `.n-plugin` (signed-only), signed sample, draft через встановлений Wasm.

## Публічний API

- `plugin_a2ui_sample_sidebar` — validated sample surface
- `plugin_sidebar_create_draft` — `create_draft_from_installed` для `com.example.mail-draft-helper` або помилка «встановіть через Manager»
- `plugin_manager_preview_install` / `plugin_manager_install` — `allow_unsigned: false`, ключ з companion `.pub`
- `plugin_manager_install_sample` / `list` / `set_disabled` / `uninstall`
