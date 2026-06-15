---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T13:34:50+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

## ADR Шар бекенду, доступний для LLM: порт n-tool-surface

## Context and Problem Statement

`mlmail` звертався до Rust/Tauri-команд безпосередньо через сирий `invoke` в `auth-store.js`. Бачення продукту (README) передбачає AI-воронку (саммері → озвучення → дії), яка потребує, щоб LLM-агент і UI виконували ті самі backend-дії незалежно. Потрібен єдиний шар, що однаково обслуговує UI та майбутній LLM-runner.

## Considered Options

* Перенести архітектуру `n-tool-surface` з `nitra/task` (єдиний каталог тулів + `dispatch` + маніфест OpenAI function-calling; UI і LLM — рівноправні адаптери)
* Залишити прямі `invoke`-виклики в `auth-store.js`

## Decision Outcome

Chosen option: "Перенести архітектуру `n-tool-surface` з `nitra/task`", because це точно відповідає бачення продукту: будь-яка дія кнопки має бути виконуваною без UI, а claude-api/локальна модель і UI є рівноправними споживачами одного каталогу.

### Consequences

* Good, because transcript фіксує очікувану користь: `catalog.js` стає єдиним джерелом правди; `dispatch` зберігає backend-`kind` (ReauthRequired), не ламаючи reauth-логіку; той самий маніфест OpenAI function-calling обслуговує майбутній LLM-runner без дублювання.
* Bad, because auth lifecycle-команди (`login`, `logout`, `getAccessToken`) лишились прямими `invoke` — вони не є тулами для LLM і не увійшли в каталог.

## More Information

Файли: `app/src/tool/catalog.js`, `dispatch.js`, `manifest.js`, `transports.js`, `index.js`, `tool.test.js`. Правило `n-tool-surface.mdc` підтягнуто через `npx @nitra/cursor`. Тул `scope: "safe"` / `"mutate"` — confirm-гейт для деструктивних дій. CLI-транспорт свідомо виключено: у `mlmail` є лише два споживачі (UI + LLM). ADR зафіксовано у `docs/adr/n-tool-surface-llm-доступний-бекенд.md`, коміт `94b77df`.

---

## ADR Локальний LLM per-platform: omlx (macOS) + LiteRT-LM (Android 16+), без CLI

## Context and Problem Statement

Для реалізації AI-воронки (саммері → озвучення → дії) потрібен LLM-провайдер. `mlmail` — кросплатформний застосунок (macOS і Android). Підхід, обраний у `nitra/task` (macOS-only, omlx локально), не вкриває Android-платформу.

## Considered Options

* Claude API через `tauri-plugin-http` (розглядався як варіант «A» в `nitra/task`, відкладений як запасний)
* omlx (OpenAI-сумісний MLX-сервер, `localhost`) — macOS
* MediaPipe LLM Inference API — Android (відкинуто: задепрекейчено Google на 2026 рік)
* LiteRT-LM — Android 16+

## Decision Outcome

Chosen option: "omlx для macOS + LiteRT-LM для Android 16+", because користувач явно зазначив: «для десктопа робимо аналогічно omlx, для Android також локальні моделі (LiteRT-LM)»; Claude API відхилено явно, MediaPipe відхилено через deprecation.

### Consequences

* Good, because transcript фіксує очікувану користь: обидві платформи залишаються offline-first; `runAgent({ chat, dispatch })` з інжектованим `chat` дозволяє підміняти LLM-адаптер не чіпаючи тул-сурфейс.
* Bad, because на Android повноцінний agent-loop (OpenAI function-calling) не гарантований — Gemini Nano/LiteRT-LM не має надійного general function-calling; старт через task-specific pipeline (summarize/rewrite) або Structured Output API. TTS — окрема нативна команда (Android `TextToSpeech` / macOS `AVSpeechSynthesizer`), не LLM.

## More Information

Transcript зазначає: MediaPipe LLM Inference API задепрекейчено Google (рекомендована міграція на LiteRT-LM); ML Kit GenAI (AICore) дає task-specific API (Summarization, Rewriting, Structured Output). CLI-транспорт повністю виключено. Архітектурно: `chat`-адаптер різний per-platform, `dispatch` однаковий. Пам'ять проєкту: `memory/tool-surface-llm-architecture.md`.

---

## ADR Міграція тестового стеку з `bun test` на Vitest

## Context and Problem Statement

Проєкт використовував `bun test` + `@happy-dom/global-registrator` (preload-хак) для JS-тестів. Синхронізація правил `@nitra/cursor` до v11 принесла vitest-орієнтовані конфіги (`app/vitest.config.mjs`, `scripts/vitest.config.mjs`, `@stryker-mutator/vitest-runner` у `stryker.config.mjs`), які конфліктували з bun-test сетапом і регенеровувалися Stop-хуком щоходу, брудячи дерево й ламаючи `knip`.

## Considered Options

* Мігрувати на Vitest (узгодити з `@nitra/cursor` v11 і `nitra/task`)
* Розширити `knip`-ignore для vitest-файлів як паліатив, лишаючи `bun test`

## Decision Outcome

Chosen option: "Мігрувати на Vitest", because користувач відповів «перемігровуємо на vitest» на запит щодо конфлікту bun-test vs v11-шаблону.

### Consequences

* Good, because `vitest.config.mjs` через `mergeConfig(vite.config.js)` дає нативну SFC-компіляцію та авто-імпорти — `happy-dom.preload.js` і хак `?initial-state` більше не потрібні; `@stryker-mutator/vitest-runner` тепер коректно listed; Stop-хук перестав регенерувати конфліктуючі файли (idempotent skip); `bun run lint` завершився з `exit 0`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Зміни: `app/vitest.config.mjs` (`mergeConfig` з callback-vite.config.js, `environment: 'happy-dom'`); `app/stryker.config.mjs` → `testRunner: 'vitest'` + `perTest`; 5 тест-файлів — `bun:test` → `vitest` (`vi.hoisted`+`vi.mock`, `vi.resetModules()` замість `?initial-state`); видалено `test/happy-dom.preload.js`, `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc`; root devDeps += `vitest`, `@vitest/coverage-v8`, `@stryker-mutator/vitest-runner`; `scripts/package.json` += `vitest`. 85/85 тестів проходять. Коміт `94b77df`.
