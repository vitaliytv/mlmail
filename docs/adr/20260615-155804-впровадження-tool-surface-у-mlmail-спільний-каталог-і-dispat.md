---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T15:58:04+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

## ADR Впровадження Tool Surface у mlmail — спільний каталог і dispatch для UI та LLM

## Context and Problem Statement

mlmail потребував єдиного шару бекенду, доступного як для UI (Tauri), так і для майбутнього on-device LLM-агента, щоб обидва споживачі викликали ті самі дії без дублювання логіки. Правило `n-tool-surface.mdc` (з `@nitra/cursor`) визначало канон, але у mlmail відсутній CLI-споживач.

## Considered Options
* Спільний `catalog.js` + `dispatch.js` + `manifest.js` з двома рівноправними адаптерами (UI та LLM), без CLI
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Спільний catalog/dispatch/manifest без CLI-споживача", because у mlmail є лише два споживачі (UI-транспорт через Tauri `invoke` і майбутній LLM-адаптер через OpenAI function-calling), а CLI-поверхня, передбачена каноном `n-tool-surface`, свідомо виключена.

### Consequences
* Good, because transcript фіксує очікувану користь: `auth-store.js` переведено з прямих `invoke` на `dispatch`; 85/85 JS-тестів проходять; manifest дає OpenAI function-calling форму для LLM-адаптера.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Файли: `app/src/tool/catalog.js`, `dispatch.js`, `manifest.js`, `transports.js`, `index.js`, `tool.test.js`. `tauriTransport` викликає `invoke(tool.tauri)` без `{}` для команд без input — для сумісності з наявними тестами (`Object.keys(input ?? {}).length === 0`). Правило підтягнуто через `npx @nitra/cursor` (синк до v11.1.0). Коміт: `16f6e7a` / `94b77df` на `main`.

---

## ADR Міграція тестового стеку з bun test на Vitest

## Context and Problem Statement

Проєкт використовував `bun test` з кастомним `happy-dom.preload.js` та хаками (`?initial-state` query-suffix для ізоляції модулів). Синк `@nitra/cursor` v11 приніс vitest-орієнтовані конфіги (`vitest.config.mjs`, `@stryker-mutator/vitest-runner`), що постійно регенерувались Stop-хуком і ламали `bun run lint` (knip: unlisted dependency, unused files). Для узгодження з v11-шаблоном та зняття churn-у вирішено мігрувати на Vitest.

## Considered Options
* Лишити bun test та ігнорувати/видаляти v11-конфіги після кожного Stop-хуку
* Мігрувати тестовий стек на Vitest (узгодження з `@nitra/cursor` v11 і проєктом `nitra/task`)

## Decision Outcome
Chosen option: "Мігрувати на Vitest", because v11-шаблон `@nitra/cursor` генерує vitest-конфіги idempotently після кожного Stop-хуку; прийняття міграції усуває churn; Vitest через `mergeConfig(vite.config.js)` дає нативну SFC-компіляцію та авто-імпорти без preload-хаків.

### Consequences
* Good, because transcript фіксує очікувану користь: `happy-dom.preload.js` і залежності `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc` видалено; `vi.resetModules()` замінює `?initial-state` трюк; knip та jscpd переходять у exit 0; 85/85 тестів проходять під vitest.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Конфіг: `app/vitest.config.mjs` (`mergeConfig` з callback vite-конфігу, `environment: 'happy-dom'`). Тест-файли конвертовано: `bun:test` → `vitest`; `mock.module` → `vi.hoisted` + `vi.mock`; `mock()` → `vi.fn()`. `app/stryker.config.mjs`: `testRunner: 'vitest'` + `perTest: true`. Залежності: root += `vitest@4.1.9`, `@vitest/coverage-v8@4.1.9`, `@stryker-mutator/vitest-runner@9.6.1`; app += `happy-dom`; scripts += `vitest`. Коміт: `94b77df` (squash) на `main`.

---

## ADR Вшивання Google Desktop OAuth client_secret у вихідний код

## Context and Problem Statement

`.env.secret` з `MLMAIL_GOOGLE_DESKTOP_CLIENT_SECRET` був відсутній, що призводило до `AuthError::OAuth` замість зрозумілого повідомлення. Google документує, що client_secret для типу **Desktop app** є «non-confidential by design» — він однаково потрапляє в бінарник і PKCE захищає флоу незалежно від нього. Стояло питання: тримати в `.env.secret` чи вшити в код.

## Considered Options
* Зберігати secret у `.env.secret` (gitignored), запікати в бінарник на CI
* Вшити secret як `const DESKTOP_SECRET_BUILTIN` у `config.rs`, додати виняток для trufflehog

## Decision Outcome
Chosen option: "Вшити secret у config.rs", because Google явно дозволяє Desktop-клієнту мати «публічний» secret; extractability з бінарника однакова в обох варіантах; вшивання усуває залежність від наявності `.env.secret` у рантаймі.

### Consequences
* Good, because transcript фіксує очікувану користь: `cargo build` компілюється, trufflehog повертає 0 знахідок (виняток у `.trufflehog-exclude`), `bun run lint` → exit 0, логін працює.
* Bad, because secret потрапив у git-історію та в transcript цього чату — при витоку репо або ротації потрібно оновлювати код. Transcript фіксує це як явно прийнятий ризик.

## More Information
Реалізація: `config.rs` — `const DESKTOP_SECRET_BUILTIN: &str = "GOCSPX-…"; // cspell:disable-line`; `desktop_client_secret()` бере env/`.env.secret` якщо задано, інакше повертає `DESKTOP_SECRET_BUILTIN`. `.trufflehog-exclude` += `app/src-tauri/src/auth/config\.rs$`. Коміт: `967482f` на `main`.

---

## ADR Гард ConfigMissing для ненастроєних OAuth credentials

## Context and Problem Statement

При порожніх або placeholder-значеннях `client_id`/`client_secret` `run_login` запускав повний OAuth-флоу, Google повертав помилку, і користувач бачив «Помилка авторизації Google.» — без підказки, що саме налаштувати. Додатково `is_real_client_id("")` повертав `true` (порожній рядок не починається з `REPLACE_ME`), тож пустий id вважався «справжнім».

## Considered Options
* Гард: перевіряти `is_real_client_id`/`is_real_client_secret` до старту флоу, повертати `AuthError::ConfigMissing(env_var)` з чітким повідомленням
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Гард ConfigMissing перед стартом флоу", because він не змінює логіку авторизації, лише додає ранню перевірку з actionable-повідомленням.

### Consequences
* Good, because transcript фіксує очікувану користь: Rust config-тести 3/3 (новий `empty_or_blank_value_is_not_considered_real`), JS-тести 86/86 (новий `ConfigMissing`), `bun run lint` → exit 0.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Файли: `app/src-tauri/src/auth/error.rs` (`AuthError::ConfigMissing(String)`), `app/src-tauri/src/auth/config.rs` (`is_real_client_id` додає перевірку `!value.trim().is_empty()`; нова функція `require_configured`), `app/src-tauri/src/auth/mod.rs` (`run_login` для macOS і Android викликає `require_configured` перед флоу), `app/src/i18n/auth-errors.js` (`ConfigMissing: 'Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.'`). Коміт: `0fdde42` на `main`.
