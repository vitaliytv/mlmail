---
type: Rust Module
title: plugin_context.rs
resource: app/src-tauri/src/plugin_context.rs
docgen:
  crc: d048e509
  model: manual
---

## Огляд

Координує durable desired context між native typed Gmail provider та встановленим Draft Helper WebAssembly Component. Відновлює committed context без мережі й не допускає Gmail emission, доки обидві exact-release instances не активні.

## Поведінка

Provider і consumer зберігаються як окремі entries, пов’язані exact WIT identity `nitra:gmail/drafts@0.1.0`. Provider replacement спочатку виводить Draft Helper із service, а повторна activation запускає provider раніше за consumer. Manual disable зберігає exact Draft Helper entry, але не активує його автоматично.

Interrupted process state ремонтується в порядку consumers → providers. Після settled activation online action виконується один раз і не реєструється як reversible effect, оскільки створення Gmail draft не має безпечного автоматичного rollback.

## Публічний API

- `draft_helper_context` — будує повний desired context для optional exact Draft Helper release та manual enablement.
- `context_database` — повертає шлях durable SQLite context у application data.
- `PluginContextCoordinator` — відновлює context, публікує replacements, gate-ить Draft Helper action і виконує graceful shutdown без видалення desired generation.

## Гарантії поведінки

- OAuth token, Gmail payload і HTTP response не потрапляють у durable metadata.
- Offline replay не виконує package resolution або Gmail request.
- Gmail emission починається тільки після active state provider і consumer.
- Desired generation переживає graceful shutdown і process restart.
