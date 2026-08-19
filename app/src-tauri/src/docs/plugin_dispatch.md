---
type: Rust Module
title: plugin_dispatch.rs
resource: app/src-tauri/src/plugin_dispatch.rs
docgen:
  crc: c6e59991
  model: manual
---

## Огляд

Визначає fail-closed boundary між typed Tauri command і exact встановленим WebAssembly Component. Dispatcher не обирає plugin за порядком у списку: caller передає повний `ReleaseIdentity` із digest.

## Поведінка

Selection звіряє committed installed projection, immutable activation generation, active pointer, lifecycle, typed trigger із product contract registry та CAS artifact. Будь-яка розбіжність release, generation, status або trigger завершує виклик до guest logic.

Durable context використовує стабільний package-scoped id `plugin:{package}`, а exact release повторно перевіряється coordinator-ом. Перед OAuth token кожен stored host interface перетворюється на product-owned capability і перевіряється exact grant для release та поточного account.

## Публічний API

- `PluginDispatchSelection` — exact Component bytes, release, generation, context id та host imports одного дозволеного виклику.
- `durable_context_id` — повертає стабільний package-scoped durable instance id.

## Гарантії поведінки

- Package без digest не є invocation target.
- Installed projection triggers не використовуються як authority; trigger береться зі stored generation.
- Unknown host capability mapping і відсутній exact grant блокують виклик до token acquisition.
- CAS bytes читаються лише після всіх exact identity та lifecycle checks.
