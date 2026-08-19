---
type: Rust Module
title: plugins.rs
resource: app/src-tauri/src/plugins.rs
docgen:
  crc: b1658b6d
  model: manual
---

## Огляд

Product boundary для preflight, встановлення, enablement, видалення та typed invocation WebAssembly Components. Поєднує product-owned contract registry, immutable activation registry і durable context, не передаючи OAuth credentials у plugin files або lifecycle metadata.

## Поведінка

Preflight читає local Component і повертає generic compatibility/consent preview без CAS, SQLite, installed projection чи activation writes. Install використовує той самий preflight service, будує dependency-free singleton graph та атомарно публікує activation generation у CAS-backed registry. Окремий durable desired context містить native Gmail provider і exact Draft Helper release; однаковий context повторно не публікується.

Disable зберігає release у UI index і durable context як manually disabled. Uninstall вилучає Draft Helper entry. Invocation відновлює або replay-ить context, чекає active provider та consumer, і лише потім отримує app-owned OAuth token та створює Gmail draft. Cleanup failure після успішної emission пишеться в log і не перетворює вже створений draft на retryable command error.

## Публічний API

- `InstalledPlugin` — UI projection exact release, triggers і manual enablement.
- `PluginDraftActionDto` — повертає opaque Gmail draft id та exact plugin release.
- `plugin_manager_list` — читає встановлені root Components.
- `plugin_manager_preflight` — повертає read-only preview local `.n-plugin` Component.
- `plugin_manager_install` — compatibility wrapper, що перевіряє через generic preflight й активує compatible local Component.
- `plugin_manager_set_disabled` — змінює manual enablement без видалення release.
- `plugin_manager_uninstall` — вилучає root plugin із desired context.
- `plugin_draft_helper_create` — виконує typed Draft Helper call після durable activation gate.

## Гарантії поведінки

- Core-Wasm, ZIP packages і unresolved dependency graphs не активуються цим installer path.
- Package identity не використовується як Draft Helper-specific allowlist.
- Unknown triggers та host imports fail closed до будь-якого activation state write.
- Component bytes читаються тільки з verified active CAS generation.
- Disabled або unavailable Draft Helper не виконує Gmail request.
- Успішна Gmail emission не повторюється через подальшу lifecycle cleanup error.
