---
type: Module
title: plugin_contracts.rs
resource: app/src-tauri/src/plugin_contracts.rs
docgen:
  crc: c9abd538
  model: manual
---

## Огляд

Product-owned registry визначає typed WIT contracts, які поточна версія `mlmail` може
зв’язати з host API та product actions. Усі interface descriptors беруть exact package
digest з одного `wkg.lock`, тому installation preflight, activation compiler, linker і
public plugin environment використовують однаковий набір identities.

## Поведінка

Registry відхиляє duplicate trigger identities, не резолвить невідомі triggers та будує
Wasmtime runtime лише з generated Gmail linker registrations. Host imports також мають
product-local consent metadata: Gmail search вимагає account-scoped `mail:search`, а
створення чернетки — account-scoped `mail:draft.create`.

## Публічний API

- `MlmailPluginContractRegistry::load` завантажує exact Gmail release з `wkg.lock`.
- `host_inventory` і `trigger_inventory` повертають inventories для activation compiler та installation preflight.
- `action_for` зіставляє exact trigger з typed product action.
- `capability_requirements_for` повертає consent requirements exact host interface.
- `build_runtime` створює runtime з того самого набору contracts.
- `environment_context` повертає public compatibility metadata поточного application release.

## Гарантії поведінки

- Domain contracts залишаються у `mlmail`, а не в `n-plugin`.
- Registry не створює dynamic JSON/string ABI.
- Host interfaces і triggers використовують один canonical WKG digest.
- Невідомий exact identity не отримує action або capability mapping.
