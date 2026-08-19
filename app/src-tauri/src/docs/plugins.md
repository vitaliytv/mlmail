---
type: Rust Module
title: plugins.rs
resource: app/src-tauri/src/plugins.rs
docgen:
  crc: c3639930
  model: manual
---

## Огляд

Product boundary для preflight, встановлення, enablement, видалення та typed invocation WebAssembly Components. Поєднує product-owned contract registry, immutable activation registry і durable context, не передаючи OAuth credentials у plugin files або lifecycle metadata.

## Поведінка

Preflight читає local Component і повертає account-bound compatibility/consent preview без CAS,
SQLite, installed projection чи activation writes. Confirm повторно читає bytes, звіряє exact
release, `preview_id` і opaque grant decisions, після чого серіалізовано публікує activation.
Activation registry є commit record; pending journal забезпечує idempotent roll-forward installed
projection і durable context після restart. Це логічна recoverable операція, а не cross-store
транзакція.

Disable зберігає release у UI index і durable context як manually disabled. Uninstall вилучає Draft Helper entry. Invocation відновлює або replay-ить context, чекає active provider та consumer, і лише потім отримує app-owned OAuth token та створює Gmail draft. Cleanup failure після успішної emission пишеться в log і не перетворює вже створений draft на retryable command error.

## Публічний API

- `InstalledPlugin` — UI projection exact release, triggers і manual enablement.
- `PluginDraftActionDto` — повертає opaque Gmail draft id та exact plugin release.
- `plugin_manager_list` — читає встановлені root Components.
- `plugin_manager_preflight` — повертає read-only preview local `.n-plugin` Component.
- `plugin_manager_confirm_install` — підтверджує exact preview та required grants.
- `plugin_manager_install` — compatibility wrapper, що перевіряє через generic preflight й активує compatible local Component.
- `plugin_manager_set_disabled` — змінює manual enablement без видалення release.
- `plugin_manager_uninstall` — вилучає root plugin із desired context.
- `plugin_draft_helper_create` — виконує typed Draft Helper call після durable activation gate.

## Гарантії поведінки

- Core-Wasm, ZIP packages і unresolved dependency graphs не активуються цим installer path.
- Package identity не використовується як Draft Helper-specific allowlist.
- Unknown triggers та host imports fail closed до будь-якого activation state write.
- Stale bytes, account або consent set вимагають нового preview.
- Registry-committed activation із незавершеною projection відновлюється через pending journal.
- Component bytes читаються тільки з verified active CAS generation.
- Disabled або unavailable Draft Helper не виконує Gmail request.
- Успішна Gmail emission не повторюється через подальшу lifecycle cleanup error.
