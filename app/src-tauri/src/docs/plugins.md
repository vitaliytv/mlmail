---
type: Rust Module
title: plugins.rs
resource: app/src-tauri/src/plugins.rs
docgen:
  crc: 67541f6b
  model: manual
---

## Огляд

Product boundary для preflight, встановлення, enablement, видалення та typed invocation WebAssembly Components. Поєднує product-owned contract registry, immutable activation registry і durable context, не передаючи OAuth credentials у plugin files або lifecycle metadata.

## Поведінка

Preflight читає local Component, online резолвить exact OCI dependencies у product-owned cache/lock
і повертає account-bound compatibility/consent preview без activation writes. Confirm повторно
читає bytes, strictly offline перевіряє той самий graph, звіряє exact release, lock, `preview_id`
і opaque grant decisions, після чого серіалізовано публікує activation з edge policy.
Activation registry є commit record; pending journal забезпечує idempotent roll-forward installed
projection і durable context після restart. Це логічна recoverable операція, а не cross-store
транзакція.

Disable та uninstall приймають exact `ReleaseIdentity`, тому update race не може змінити інший digest того самого package. Uninstall переводить generation у durable retirement, оновлює context, а потім запускає registry-owned quarantine/grace GC, який зберігає shared reachable dependencies. Typed commands авторизують exact grants та dependency edges до OAuth і повторюють edge check під час Component instantiation.

## Публічний API

- `InstalledPlugin` — durable product index exact release, deployment lock, grants, generation і manual enablement.
- `InstalledPluginDto` — output-only UI projection exact generation, dependencies, lock, registry lifecycle і product-owned actions.
- `PluginDraftActionDto` — повертає opaque Gmail draft id, exact plugin release та generation.
- `PluginBookingActionDto` — повертає typed booking query, message references, exact release та generation.
- `plugin_manager_list` — читає встановлені root Components.
- `plugin_manager_preflight` — повертає read-only preview local `.n-plugin` Component.
- `plugin_manager_confirm_install` — підтверджує exact preview та required grants.
- `plugin_manager_install` — compatibility wrapper, що перевіряє через generic preflight й активує compatible local Component.
- `plugin_manager_set_disabled` — змінює manual enablement без видалення release.
- `plugin_manager_uninstall` — вилучає root plugin із desired context.
- `plugin_draft_helper_create` — виконує typed Draft Helper call після durable activation gate.
- `plugin_booking_finder_find` — виконує typed Booking Finder call без dynamic JSON broker.

## Гарантії поведінки

- Core-Wasm, ZIP packages і unresolved або digest-mismatched dependency graphs не активуються цим installer path.
- Package identity не використовується як Draft Helper-specific allowlist.
- Unknown triggers та host imports fail closed до будь-якого activation state write.
- Stale bytes, account або consent set вимагають нового preview.
- Registry-committed activation із незавершеною projection відновлюється через pending journal.
- Component bytes читаються тільки з verified active CAS generation.
- Wrong digest, disabled, uninstalled або stale generation не виконують guest logic чи Gmail request.
- Exact generic `mail:draft.create` або `mail:search` grant перевіряється до OAuth token.
- Online transport закінчується на preview; confirm не звертається до registry/network.
- Успішна Gmail emission не повторюється через подальшу lifecycle cleanup error.
