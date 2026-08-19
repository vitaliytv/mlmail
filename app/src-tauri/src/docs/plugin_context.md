---
type: Rust Module
title: plugin_context.rs
resource: app/src-tauri/src/plugin_context.rs
docgen:
  crc: 3478d94e
  model: manual
---

## Огляд

Координує durable desired context між native typed Gmail provider та всіма exact встановленими WebAssembly Components. Відновлює committed context без мережі й не допускає Gmail emission, доки provider та обрана exact-release instance не активні.

## Поведінка

Provider і plugins зберігаються як окремі entries. Native provider публікує typed Gmail drafts і search interfaces; app-invoked roots лише вимагають свої stored host interfaces і не оголошують duplicate trigger providers. Manual disable зберігає exact entry, але не активує його автоматично.

Interrupted process state ремонтується в порядку consumers → providers. Після settled activation online action виконується один раз і не реєструється як reversible effect, оскільки створення Gmail draft не має безпечного автоматичного rollback.

## Публічний API

- `draft_helper_context` — compatibility helper для одного exact Draft Helper release.
- `PluginContextEntry` — описує stable context id, exact release, host imports і manual enablement встановленого root.
- `plugin_context` — будує повний desired context для native provider та довільного набору installed roots.
- `context_database` — повертає шлях durable SQLite context у application data.
- `PluginContextCoordinator` — відновлює context, публікує replacements, gate-ить action за active state та exact desired release і виконує graceful shutdown.

## Гарантії поведінки

- OAuth token, Gmail payload і HTTP response не потрапляють у durable metadata.
- Offline replay не виконує package resolution або Gmail request.
- Gmail emission починається тільки після active state provider і exact selected plugin.
- Package-scoped context id не послаблює identity: coordinator додатково звіряє повний `ReleaseIdentity`.
- Desired generation переживає graceful shutdown і process restart.
