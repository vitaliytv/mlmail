# Розробка плагінів для mlmail

<!-- cspell:words Forgejo oras -->

## Межа платформи й продукту

`n-plugin` збирає, перевіряє, публікує та фіксує exact WebAssembly Components. `mlmail`
визначає typed WIT contracts, consent і окремі product actions. Новий Component, який реалізує
вже підтриманий contract, можна встановити без нового build застосунку. Новий WIT contract
спочатку потребує product-local реалізації в `mlmail`.

Повний опис формату package, OCI transport і lock/cache semantics містить
[канонічний посібник n-plugin](https://git.7n.ai/nitra/n-plugin/src/branch/main/docs/plugin-system.md).
Цей документ описує лише інтеграцію з `mlmail`.

## Підтримані contracts

| Product action | Trigger export | Host import | Account capability |
| --- | --- | --- | --- |
| Create draft | `nitra:gmail/draft-helper@0.1.0` | `nitra:gmail/drafts@0.1.0` | `mail:draft.create` |
| Find Booking messages | `nitra:gmail/booking-finder@0.1.0` | `nitra:gmail/search@0.1.0` | `mail:search` |

Ідентичності мають збігатися точно. Невідомий trigger, host import або непідтримувана версія
відхиляються під час preflight до activation. `mlmail` не має універсального JSON/string ABI:
кожна дія виконується через generated typed adapter.

## Передумови

- Rust stable і target `wasm32-wasip2`;
- `n-plugin` у `PATH` або шлях у `N_PLUGIN_BIN`;
- `jq` для локальної test matrix;
- `oras login git.7n.ai` лише для publish до private Forgejo namespace.

```bash
rustup target add --toolchain stable wasm32-wasip2
n-plugin profile
```

## Створення Component

`n-plugin new` створює generic Rust Component, а не готовий Gmail plugin. Його можна використати
як початкову структуру, але WIT world, Rust bindings і `.n-plugin.toml` треба привести до одного
з contracts у таблиці. Практичні product templates є в:

- `app/src-tauri/plugins/draft-helper`;
- `app/src-tauri/plugins/booking-finder`.

```bash
n-plugin new my-draft-helper \
  --publisher vitaliytv \
  --package my-draft-helper \
  --version 0.1.0
```

Для Draft Helper manifest має оголошувати typed trigger та entrypoint:

```toml
schema = "nitra.plugin-manifest/v1"
publisher_id = "vitaliytv"
package = "my-draft-helper"
version = "0.1.0"

triggers = ["nitra:gmail/draft-helper@0.1.0"]

[entrypoints]
create = "nitra:gmail/draft-helper@0.1.0"
```

Guest Component імпортує `nitra:gmail/drafts@0.1.0`; OAuth token, Gmail endpoint і account data
залишаються в Rust host застосунку.

## Build, package та inspect

Збери raw Component, потім вбудуй manifest. Не передавай raw `.wasm` у Plugin Manager.

```bash
cargo build --release --target wasm32-wasip2
n-plugin build \
  --manifest .n-plugin.toml \
  --component target/wasm32-wasip2/release/my_draft_helper.wasm \
  --output target/my-draft-helper.n-plugin
n-plugin inspect target/my-draft-helper.n-plugin
```

`inspect` повертає exact `package`, `version` і SHA-256 digest. Зберігай digest разом із release:
package або version без digest недостатні для invocation та update race protection.

## Required dependencies

M0 підтримує лише required dependencies з exact version. Optional dependencies і SemVer ranges
для deployable Components не підтримуються.

```toml
[dependencies.mail-helper]
package = "other-publisher:mail-helper"
requirement = "=1.2.0"
imports = ["other-publisher:mail-helper/api@1.2.0"]
```

Online lock завантажує відсутні exact dependencies, перевіряє embedded identity і digest та
записує cache. Offline lock не звертається до registry й вимагає всі pinned bytes локально.

```bash
n-plugin lock target/my-draft-helper.n-plugin \
  --registry git.7n.ai \
  --lock-file target/my-draft-helper.n-plugin.lock \
  --cache target/n-plugin-cache
n-plugin lock target/my-draft-helper.n-plugin \
  --registry git.7n.ai \
  --lock-file target/my-draft-helper.n-plugin.lock \
  --cache target/n-plugin-cache \
  --offline
```

Dependency іншого publisher є нормальною. Її package identity не залежить від OCI reference.
Dependency без host capability import не створює додаткового consent. Host capability dependency
отримує власний explicit edge-scoped requirement.

## Publish і fetch

OCI є transport, а embedded package identity та digest лишаються authoritative.

```bash
oras login git.7n.ai
n-plugin publish target/my-draft-helper.n-plugin --registry git.7n.ai
n-plugin fetch vitaliytv:my-draft-helper@0.1.0 \
  --registry git.7n.ai \
  --output target/my-draft-helper.n-plugin
```

Не записуй registry password або token у manifest, lock, документацію чи test fixtures. Для public
package fetch може працювати анонімно; publish використовує локальний ORAS/Docker-compatible
credential store.

## Installation і consent

1. Plugin Manager читає local `.n-plugin` і виконує preflight без activation writes.
2. Preview показує exact release, supported actions, dependencies та account-bound requirements.
3. UI повертає лише opaque `requirementId` і рішення allow/deny.
4. Backend повторно читає Component bytes, звіряє preview, account, contract fingerprint та exact
   release, після чого активує graph.

`mail:search` і `mail:draft.create` deny-by-default. Grant належить exact release, host edge і
поточному account; інший digest або account потребує нового consent. Query, mail body, OAuth token
і Gmail response не входять до grant records.

Якщо два Components реалізують один trigger, обидва можуть бути встановлені одночасно. UI та typed
command завжди передають exact `ReleaseIdentity`, тому runtime не обирає перший package зі списку.

## Disable, restart і uninstall

- Disable зберігає Component встановленим, але exact invocation fail-closed.
- Enable відновлює activation лише для вибраного exact release.
- Uninstall прибирає root із desired context; shared dependency залишається, доки її досягає інший
  active root.
- Звичайний restart читає active generation, composed CAS і SQLite context без resolver або lock.
  Exact deployment lock потрібен лише для offline recompose/repair; registry network для цього не
  потрібна, якщо всі pinned dependency bytes уже є в cache.

Missing або tampered cache bytes, stale generation чи digest mismatch не перемикають application на
частково активований graph. Попередня committed generation залишається authoritative.

## Локальна перевірка

Focused scripts збирають, пакують, перевіряють identity та запускають typed adapter:

```bash
bun run --cwd app test:draft-helper-component
bun run --cwd app test:booking-finder-component
```

Deterministic matrix створює Draft packages двох publishers, запускає Booking Finder, preflight,
consent, exact dispatch, dependency graph, negative compatibility та offline restart tests. Вона не
звертається до OCI за замовчуванням.

```bash
N_PLUGIN_BIN=/path/to/n-plugin \
  bun run --cwd app test:installed-plugin-matrix
```

Справжній OCI smoke test є окремим opt-in. Він тільки fetch/lock-ить заздалегідь опублікований
public fixture, вимагає exact digest і не виконує publish:

```bash
MLMAIL_PLUGIN_E2E_RELEASE='vitaliytv:mlmail-e2e-root@0.1.0' \
MLMAIL_PLUGIN_E2E_DIGEST='sha256:<64-hex-digest>' \
N_PLUGIN_BIN=/path/to/n-plugin \
  bun run --cwd app test:installed-plugin-matrix:oci
```

## Діагностика

| Помилка | Дія |
| --- | --- |
| `N_PLUGIN_BIN must point to an executable` | Збери CLI або передай правильний absolute path. |
| `wasm32-wasip2 is missing` | Виконай `rustup target add --toolchain stable wasm32-wasip2`. |
| `plugin has no entrypoint supported by this mlmail release` | Звір exact trigger/entrypoint із таблицею contracts. |
| `grant-required` | Повтори preflight для поточного account і підтвердь потрібний consent. |
| `offline resolution requires an existing plugin lock` | Один раз виконай online `n-plugin lock`. |
| `cached Component ... does not match its content digest` | Не використовуй пошкоджений cache; повторно fetch exact release online. |
| `OCI credentials ... unavailable` | Виконай `oras login <registry>` для private package. |

Durable activation і repair детальніше описані в
[контексті runtime плагінів](./plugin-context-runtime.md).
