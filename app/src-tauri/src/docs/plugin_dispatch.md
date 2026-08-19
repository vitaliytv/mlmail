---
type: Rust Module
title: plugin_dispatch.rs
resource: app/src-tauri/src/plugin_dispatch.rs
docgen:
  crc: d69e4fe9
  model: manual
---

## Огляд

Визначає fail-closed boundary між typed Tauri command і exact встановленим WebAssembly Component. Dispatcher не обирає plugin за порядком у списку: caller передає повний `ReleaseIdentity` із digest.

## Поведінка

Selection звіряє committed installed projection, immutable activation generation, active pointer, lifecycle, typed trigger із product contract registry та CAS artifact. Будь-яка розбіжність release, generation, status або trigger завершує виклик до guest logic.

Durable context використовує стабільний package-scoped id `plugin:{package}`, а exact release повторно перевіряється coordinator-ом. Перед OAuth token dispatcher перевіряє збережені root/dependency grants, незалежно від них повторно виводить root-host requirements і авторизує всі generation-scoped dependency edges. Measured WASI interfaces не створюють consent і не можуть бути записані як product grant.

## Публічний API

- `PluginDispatchSelection` — exact Component bytes, release, generation, context id, grants та edge guards одного дозволеного виклику.
- `durable_context_id` — повертає стабільний package-scoped durable instance id.

## Гарантії поведінки

- Package без digest не є invocation target.
- Installed projection triggers не використовуються як authority; trigger береться зі stored generation.
- Unknown host capability mapping і відсутній exact grant блокують виклик до token acquisition.
- CAS bytes читаються лише після всіх exact identity та lifecycle checks.
- Dependency edge policy перевіряється до OAuth acquisition і повторно linker guard-ом під час instantiation.
