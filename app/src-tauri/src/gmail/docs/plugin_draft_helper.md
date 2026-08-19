---
type: Rust Module
title: plugin_draft_helper.rs
resource: app/src-tauri/src/gmail/plugin_draft_helper.rs
docgen:
  crc: ffffa0c0
  model: manual
---

## Огляд

Generated typed adapter між Draft Helper Component world і native Gmail drafts host.

## Поведінка

Adapter компілює перевірені Component bytes і створює instance у product runtime, підключає typed Gmail/WASI imports та повертає generated draft reference. Exact release, lifecycle і generic `mail:draft.create` grant перевіряє product dispatcher до передачі OAuth token.

## Публічний API

- `invoke_draft_helper` — викликає typed `create` export і повертає opaque Gmail draft reference.

## Гарантії поведінки

- Dynamic JSON broker не використовується.
- OAuth token зберігається лише в host state на час invocation.
- Відсутній exact generic grant не може досягти adapter через Tauri command surface.
