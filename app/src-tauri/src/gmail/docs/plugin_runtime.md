---
type: Module
title: plugin_runtime.rs
resource: app/src-tauri/src/gmail/plugin_runtime.rs
docgen:
  crc: b1b5fe60
  model: manual
---

## Огляд

Gmail runtime adapter завантажує product-owned `MlmailPluginContractRegistry` і створює
Component runtime разом із public compatibility metadata. Adapter зберігає стабільні
descriptor helpers для Gmail integrations, але більше не складає host interfaces і
triggers вручну.

## Поведінка

`build_gmail_plugin_runtime` читає один exact `wkg.lock`, просить registry побудувати typed
runtime і генерує environment metadata з registry-owned application context. Descriptor
helpers отримують identity через той самий registry, тому не утворюють другого package
resolver або паралельного inventory.

## Публічний API

- `GmailPluginRuntime` надає готовий typed runtime та його public environment.
- `gmail_search_descriptor` і `gmail_drafts_descriptor` повертають exact host descriptors.
- `gmail_booking_finder_descriptor` і `gmail_draft_helper_descriptor` повертають exact trigger descriptors.
- `build_gmail_plugin_runtime` створює узгоджений runtime/environment pair.

## Гарантії поведінки

- Linker registrations та advertised host interfaces походять з одного registry.
- Trigger inventory використовує той самий locked package digest.
- Інший або відсутній Gmail release відхиляється до побудови runtime.
