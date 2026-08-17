---
type: Rust Module
title: plugins.rs
resource: app/src-tauri/src/plugins.rs
docgen:
  crc: c5c41d0e
  model: manual
---

## Огляд

Product boundary для встановлення, enablement, видалення та typed invocation WebAssembly Components. Поєднує immutable activation registry із durable context, не передаючи OAuth credentials у plugin files або lifecycle metadata.

## Поведінка

Install перевіряє embedded manifest, exact Draft Helper trigger, host inventory та Component format, після чого атомарно публікує activation generation у CAS-backed registry. Окремий durable desired context містить native Gmail provider і exact Draft Helper release; однаковий context повторно не публікується.

Disable зберігає release у UI index і durable context як manually disabled. Uninstall вилучає Draft Helper entry. Invocation відновлює або replay-ить context, чекає active provider та consumer, і лише потім отримує app-owned OAuth token та створює Gmail draft. Cleanup failure після успішної emission пишеться в log і не перетворює вже створений draft на retryable command error.

## Публічний API

- `InstalledPlugin` — UI projection exact release, triggers і manual enablement.
- `PluginDraftActionDto` — повертає opaque Gmail draft id та exact plugin release.
- `plugin_manager_list` — читає встановлені root Components.
- `plugin_manager_install` — перевіряє й активує local `.n-plugin` Component.
- `plugin_manager_set_disabled` — змінює manual enablement без видалення release.
- `plugin_manager_uninstall` — вилучає root plugin із desired context.
- `plugin_draft_helper_create` — виконує typed Draft Helper call після durable activation gate.

## Гарантії поведінки

- Core-Wasm, ZIP packages і dependency graphs не приймаються цим installer path.
- Component bytes читаються тільки з verified active CAS generation.
- Disabled або unavailable Draft Helper не виконує Gmail request.
- Успішна Gmail emission не повторюється через подальшу lifecycle cleanup error.
