---
type: Rust Module
title: lib.rs
resource: app/src-tauri/mlmail-plugin-host/src/lib.rs
docgen:
  crc: b1123b97
---

## Огляд

Domain host mlmail для nitra-плагінів: `GmailMailHost` (metadata-only Gmail HTTP) і `MailPluginSession` (grants + sample Wasm `read_meta`). Capability `mail:metadata.read` enforce через `GrantGatedMailHost`.

## Публічний API

- `GmailMailHost`, `MailPluginSession`
- `grant_metadata_message`, `load_sample_reader`, `read_meta_via_sample`
