---
type: Module
title: plugin_grants.rs
resource: app/src-tauri/src/plugin_grants.rs
docgen:
  crc: 3fa79fc1
  model: manual
---

## Огляд

Зберігає product-owned authorization для exact typed plugin-to-host edges. Grant зв’язаний із
root і subject release, host interface, capability, logical edge та sealed account/application
scope; tokens, Gmail query і message body не можуть бути частиною формату.

## Поведінка

Store читає deny-by-default JSON projection, порівнює повний exact key і публікує оновлення через
staged file, `sync_all` та atomic rename. Failed save відновлює попередній in-memory grant set.
Account-scoped key приймається лише коли public account identity збігається із sealed scope.
Resolved scope конвертується в payload-free runtime `GrantScope`; unresolved account scope
ніколи не може потрапити в immutable edge policy.

## Публічний API

- `PluginGrantScope` — sealed payload-free account або application scope.
- `PluginGrantScope::to_runtime_scope` — будує generic runtime scope без request constraints.
- `PluginGrantKey` — exact authorization key для одного typed host edge.
- `PluginGrantStore` — durable exact grant storage із deny-by-default `require`.
- `grant_store_path` — canonical application-local path `n-plugin/grants.json`.

## Гарантії поведінки

- Grant іншого digest, account, interface або capability не авторизує call.
- Arbitrary JSON constraints і request payloads не підтримуються.
- Duplicate grants детерміновано сортуються та усуваються.
