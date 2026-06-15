---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T14:03:54+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

## ADR Перенесення n-tool-surface у mlmail: спільний каталог тулів для UI та LLM

## Context and Problem Statement

Бачення mlmail (README) передбачає AI-воронку (summarize → speak → draft_reply → delete/filter/save) поверх наявного Gmail/auth бекенду, але фронтенд бив у Rust напряму через сирий `invoke` в `auth-store.js`. Не було жодного механізму, який дозволив би LLM-агенту виконати ту саму дію, що й UI-кнопка.

## Considered Options

* Перенести n-tool-surface із `nitra/task`: єдиний каталог тулів + `dispatch` + `manifest` — UI, LLM і (можливо) оркестратор як рівноправні адаптери.
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Перенести n-tool-surface з nitra/task", because архітектура вже перевірена (34/34 тести в `task`), і млmail тепер залежить від `@nitra/cursor` з правилом `n-tool-surface.mdc`, яке авто-активується на `@tauri-apps/api`.

### Consequences

* Good, because transcript фіксує очікувану користь: будь-яку Gmail-дію (inbox_count, random_message, random_newsletter, unsubscribe) тепер можна виконати без UI через єдиний `dispatch`; manifest готовий для OpenAI function-calling.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Нові файли: `app/src/tool/catalog.js`, `dispatch.js`, `manifest.js`, `transports.js`, `index.js`, `tool.test.js`. `auth-store.js` переведено з прямих `invoke` на `dispatch` для read/action-команд; lifecycle-команди (`login`, `logout`, `getAccessToken`) лишились прямими. Dispatch зберігає backend-`kind` у конверті `{ok, error, errorKind}`, щоб не зламати ReauthRequired-логіку. Правило отримано через `npx @nitra/cursor`. CLI-транспорт (`bin/task.mjs`) свідомо виключено.

---

## ADR Стратегія LLM на обох платформах: локальні моделі, без CLI

## Context and Problem Statement

mlmail кросплатформний (macOS + Android), тоді як `nitra/task` обрав macOS-only + omlx (локальний MLX-сервер). Потрібно було визначити LLM-провайдера для Android і вирішити, чи переносити CLI-транспорт.

## Considered Options

* Десктоп — omlx (OpenAI-сумісний MLX-сервер, localhost), Android — LiteRT-LM.
* Десктоп — omlx, Android — ML Kit GenAI API (AICore / Gemini Nano).
* Claude API через `tauri-plugin-http` (кросплатформний хмарний варіант).
* CLI-транспорт (per-verb spawn, як у `task`).

## Decision Outcome

Chosen option: "Десктоп — omlx, Android — LiteRT-LM; CLI виключено", because користувач сформулював це явно: «для десктопа робимо аналогічно omlx, для Android також робимо локальні моделі» і «для Android використовуватимемо LiteRT-LM»; «cli не потрібен».

### Consequences

* Good, because transcript фіксує очікувану користь: оффлайн-робота на обох платформах без хмарного провайдера; той самий `dispatch` (тул-сурфейс) на обох; `runAgent` з ін'єкцією `chat` дозволяє підміняти адаптер per-platform.
* Bad, because transcript фіксує архітектурне ускладнення: Tauri Kotlin-плагін для LiteRT-LM (Android) є новим обсягом, якого не було в `task`; ML Kit GenAI не має надійного загального function-calling — на Android агент-loop обмежений Structured Output API або фіксованим пайплайном.

## More Information

Дослідження у transcript: MediaPipe LLM Inference API задепрекейчено на Android, Google рекомендує LiteRT-LM. ML Kit GenAI Summarization/Rewriting — task-specific API поверх AICore. TTS (`speak`) — окрема нативна команда (Android `TextToSpeech` / macOS `AVSpeechSynthesizer`), не LLM. Правило `n-tool-surface.mdc` v1.0 передбачає CLI як опційного третього адаптера; mlmail свідомо його пропускає.

---

## ADR Міграція тестового стеку з bun:test на Vitest

## Context and Problem Statement

`@nitra/cursor` оновлено з v1.27.5 до v11.1.0 (sync під час впровадження n-tool-surface). v11 генерує `vitest.config.mjs` і `stryker.config.mjs` з `vitest-runner` — ці файли конфліктували з наявним bun-test стеком, регенерувалися Stop-хуком після кожного ходу і спричиняли knip-фейли.

## Considered Options

* Мігрувати на Vitest (узгодити з v11 і `nitra/task`).
* Лишити bun:test і розширити knip-ignore на vitest-файли як паліатив.

## Decision Outcome

Chosen option: "Мігрувати на Vitest", because користувач сформулював явно: «перемігровуємо на vitest».

### Consequences

* Good, because transcript фіксує очікувану користь: v11 sync-конфіги тепер коректні й ідемпотентно пропускаються хуком (churn зник); SFC-компіляція через Vite-плагіни нативна — `happy-dom.preload.js` не потрібен; `stryker.config.mjs` → `vitest-runner` + `perTest`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Нові залежності: root — `vitest`, `@vitest/coverage-v8`, `@stryker-mutator/vitest-runner`; `app` — `happy-dom`; `scripts` — `vitest`. Видалено: `app/test/happy-dom.preload.js`, `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc`. `app/vitest.config.mjs` використовує `mergeConfig(viteConfig(), …)` + `environment: 'happy-dom'` (vite.config.js — callback, тому mergeConfig загорнуто у функцію). 5 тест-файлів переписано: `bun:test` → `vitest`; `mock.module` → `vi.hoisted` + `vi.mock`; `?initial-state` трюк → `vi.resetModules()`. Команди `test*` → `vitest run/watch/coverage`. 85/85 тестів зелені.

---

## ADR Гард `ConfigMissing` для незаповнених OAuth-credentials

## Context and Problem Statement

`app/src-tauri/.env` містив порожні значення `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID=`. Функція `is_real_client_id("")` повертала `true` (порожній рядок не починається з `REPLACE_ME`), тому login-флоу стартував із порожнім client_id, Google повертав помилку, а користувач бачив «Помилка авторизації Google.» без жодної вказівки на причину.

## Considered Options

* Додати явний варіант помилки `ConfigMissing` і гард у `run_login` перед стартом флоу.
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Додати ConfigMissing гард", because користувач підтвердив явно: «Зробити цей гард».

### Consequences

* Good, because transcript фіксує очікувану користь: замість плутаного «Помилка авторизації Google» користувач бачить «Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.»; `is_real_client_id` тепер відхиляє порожній і пробільний рядки.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінені файли: `app/src-tauri/src/auth/error.rs` (новий variant `ConfigMissing(String)`), `app/src-tauri/src/auth/config.rs` (`is_real_client_id` — додано `!value.trim().is_empty()` + тест `empty_or_blank_value_is_not_considered_real`), `app/src-tauri/src/auth/mod.rs` (гард у `run_login` для `macos` та `android` — повертає `Err(AuthError::ConfigMissing(…))` якщо `!is_real_client_id`), `app/src/i18n/auth-errors.js` (новий ключ `ConfigMissing`), `app/src/i18n/auth-errors.test.js` (новий тест). Rust config-тести: 3/3 ok.
