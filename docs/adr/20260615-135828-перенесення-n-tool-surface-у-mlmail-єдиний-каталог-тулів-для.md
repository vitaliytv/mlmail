---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T13:58:28+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

## ADR Перенесення n-tool-surface у mlmail: єдиний каталог тулів для UI і LLM

## Context and Problem Statement

mlmail має готовий бекенд (Gmail API + auth, 6 Tauri-команд), але UI звертається до кожної команди прямим `invoke` в `auth-store.js`. Архітектурне бачення README передбачає AI-воронку (читання → саммері → дії), якої неможливо реалізувати без шару, доступного однаково UI і on-device LLM-агенту. Проєкт `nitra/task` вже реалізував таку архітектуру під назвою `n-tool-surface`.

## Considered Options

* Перенести `n-tool-surface` з `nitra/task` (`catalog / dispatch / manifest / tauriTransport`), виключивши CLI-споживача
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Перенести n-tool-surface без CLI-транспорту", because у mlmail є рівно два споживачі (UI + LLM), а CLI (`bin/task.mjs`, per-verb spawn, MCP-stdio) відсутній за дизайном — Tauri-команди є єдиним бекендом.

### Consequences

* Good, because transcript фіксує очікувану користь: `catalog / dispatch / manifest` стають єдиним джерелом правди; будь-яку дію кнопки можна виконати без UI через той самий `dispatch`; `validateInput` захищає виклик ще до транспорту; уніфікований конверт `{ok, output}` / `{ok: false, error, kind}` зберігає `kind` (зокрема `ReauthRequired`) для reauth-логіки фронтенду.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файли, створені в сесії: `app/src/tool/catalog.js`, `dispatch.js`, `manifest.js`, `transports.js`, `index.js`, `tool.test.js`. `auth-store.js` переписано — read/action-команди (`inbox_count`, `random_message`, `random_newsletter`, `unsubscribe`, `is_authenticated`, `current_email`) тепер через `dispatch`; lifecycle-команди (`auth_start_login`, `auth_complete_login`, `auth_logout`, `auth_get_access_token`) залишились прямим `invoke`. Правило `n-tool-surface.mdc` підтягнуто через `npx @nitra/cursor` (v11.1.0). Коміт: `94b77df`.

---

## ADR Локальні LLM per-platform: omlx (macOS) + LiteRT-LM (Android 16+)

## Context and Problem Statement

Для реалізації AI-шару (саммері листа, класифікація дій, чернетки відповіді) необхідно обрати LLM-провайдера. mlmail кросплатформний (macOS + Android), тому рішення має покривати обидві платформи. Альтернатива хмарного API (Claude API через `tauri-plugin-http`) розглядалась, але не була обрана.

## Considered Options

* Локальні моделі на обох платформах: omlx (macOS) + LiteRT-LM (Android 16+)
* Claude API через `tauri-plugin-http` (кросплатформний хмарний варіант)

## Decision Outcome

Chosen option: "Локальні моделі на обох платформах", because user явно вказав: «для десктопа робимо аналогічно omlx, для Android також робимо локальні моделі» і «для Android використовуватимемо LiteRT-LM».

### Consequences

* Good, because transcript фіксує очікувану користь: відсутність мережевої залежності для AI-функцій; для desktops — повноцінний OpenAI-сумісний function-calling через omlx (той самий `createOpenAiChat`-адаптер, що в `nitra/task`); для Android — LiteRT-LM підтримує Gemma 3n E2B/E4B та AI-інтерференцію на-пристрої.
* Bad, because transcript фіксує архітектурну складність: на Android LiteRT-LM не надає нативного function-calling у форматі OpenAI — потребує Kotlin-плагіна Tauri, Structured Output API або фіксованого пайплайну замість вільного агент-loop; MediaPipe LLM Inference API задепрекейчено (Google рекомендує LiteRT-LM).

## More Information

Агент-loop (`runAgent` + `createOpenAiChat`) планується спершу для macOS через `omlx` (MLX-сервер, `localhost`). Android-гілка: Tauri Kotlin-плагін → LiteRT-LM → `chat`-адаптер; старт через task-specific пайплайн (summarize → classify → propose), вільний агент-loop — пізніше. TTS (`speak`) — окрема Tauri-команда через нативний синтез (Android `TextToSpeech` / macOS `AVSpeechSynthesizer`), не LLM. Web-пошук під час сесії: [Gemini Nano | Android Developers](https://developer.android.com/ai/gemini-nano), [LLM Inference guide for Android (LiteRT-LM)](https://ai.google.dev/edge/mediapipe/solutions/genai/llm_inference/android).

---

## ADR Міграція тестового стеку з bun:test на Vitest

## Context and Problem Statement

Проєкт використовував `bun test` з `@happy-dom/global-registrator`-preload для тестування Vue-компонентів. Після оновлення `@nitra/cursor` до v11.1.0 (через `npx @nitra/cursor` для отримання правила `n-tool-surface.mdc`) Stop-хук почав регенеровувати `app/vitest.config.mjs` та `scripts/vitest.config.mjs`, що ламало `knip` і `jscpd`. Необхідно було вирішити: відкотити апгрейд або мігрувати на vitest.

## Considered Options

* Мігрувати на Vitest (відповідно до канону `@nitra/cursor` v11 і стеку `nitra/task`)
* Відкотити `@nitra/cursor` до `^1.27.5` і лишити `bun test`

## Decision Outcome

Chosen option: "Мігрувати на Vitest", because user явно сказав «перемігровуємо на vitest» у відповідь на виявлений конфлікт.

### Consequences

* Good, because transcript фіксує очікувану користь: виключено bun-специфічні хаки (`happy-dom.preload.js`, `?initial-state`-трюк, `@types/bun`); `vitest.config.mjs` наслідує `vite.config.js`-плагіни (Vue/VueMacros/AutoImport/Quasar) → нативна SFC-компіляція та авто-імпорти в тестах без окремого setup; `stryker.config.mjs` переходить на `vitest-runner` + `perTest` (прогін тільки дотичних тестів); `bun run lint` → exit 0 після міграції; 85/85 тестів зелені.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Ключові зміни: `app/vitest.config.mjs` (`mergeConfig(vite.config.js callback, …)` + `environment: 'happy-dom'`); 5 тест-файлів — `bun:test` → `vitest` (`vi.hoisted`+`vi.mock` для `@tauri-apps/api/core`-моку; `vi.resetModules()` замість `?initial-state`); видалено `app/test/happy-dom.preload.js`, `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc`; root `devDependencies` += `vitest`, `@vitest/coverage-v8`, `@stryker-mutator/vitest-runner`; `knip.json` += workspace `scripts` (entry `docs-regen.js`); `.jscpd.json` += `docs/superpowers/**`, `.cursor/**`. Коміт: `94b77df`.
