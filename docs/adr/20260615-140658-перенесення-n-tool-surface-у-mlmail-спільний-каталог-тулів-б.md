---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T14:06:58+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

## ADR Перенесення n-tool-surface у mlmail: спільний каталог тулів без CLI-адаптера

## Context and Problem Statement

У mlmail фронтенд звертався до Rust-бекенду через прямий `invoke` у `auth-store.js`. Це ускладнювало майбутнє підключення LLM-агента, оскільки не було спільного шару між UI та LLM-консумером.

## Considered Options

* Перенести `n-tool-surface` із `nitra/task` цілком (catalog / dispatch / manifest / tauriTransport / llm / bin)
* Перенести лише ядро (catalog / dispatch / manifest / tauriTransport) без CLI-бінарника
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Перенести лише ядро без CLI-адаптера", because у mlmail бекенд — суто Tauri-команди (Rust → Gmail HTTP), окремого бінарника немає; CLI-транспорт (`bin/task.mjs`, per-verb spawn) виключено свідомо.

### Consequences

* Good, because transcript фіксує очікувану користь: UI і майбутній LLM-агент стають рівноправними консумерами одного `dispatch`; `auth-store.js` переведено з прямих `invoke` на `dispatch()`; паритет UI↔LLM досягається без окремого процесу.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файли: `app/src/tool/catalog.js`, `dispatch.js`, `manifest.js`, `transports.js`, `index.js`, `tool.test.js`. `auth-store.js` переписано; lifecycle-команди (`login`, `logout`, `getAccessToken`) лишилися прямими `invoke`. Правило `n-tool-surface.mdc` підтягнуто через `npx @nitra/cursor` (v11.1.0). 85/85 JS-тестів проходять.

---

## ADR Вибір LLM-провайдера: локальні моделі per-platform без хмарного API

## Context and Problem Statement

mlmail — кросплатформний застосунок (macOS + Android). Майбутній LLM-агент (summarize / draft_reply / classify) потребує рішення, де запускати модель: локально на пристрої або через хмарний API.

## Considered Options

* Claude API через `tauri-plugin-http` (кросплатформний хмарний варіант)
* Локально per-platform: omlx на macOS + MediaPipe LLM Inference API на Android
* Локально per-platform: omlx на macOS + LiteRT-LM на Android

## Decision Outcome

Chosen option: "Локально per-platform: omlx (macOS) + LiteRT-LM (Android)", because користувач явно вибрав локальні моделі на обох платформах; MediaPipe LLM Inference API задепрекейчено Google (рекомендована заміна — LiteRT-LM).

### Consequences

* Good, because transcript фіксує очікувану користь: відсутність мережевої залежності для LLM; omlx дає OpenAI-сумісний інтерфейс, тож існуюча логіка `runAgent` з `nitra/task` переноситься напряму; LiteRT-LM підтримує Android 16+.
* Bad, because transcript фіксує обмеження: ML Kit GenAI / Gemini Nano на Android не має надійного загального function-calling → агент-loop на Android потребує окремого дизайну (Structured Output API або фіксований пайплайн); потрібен Tauri Kotlin-плагін для bridging LiteRT-LM.

## More Information

Відмовлено від Claude API через відсутність потреби в хмарі. MediaPipe задепрекейчено згідно з [LLM Inference guide for Android (MediaPipe → LiteRT-LM)](https://ai.google.dev/edge/mediapipe/solutions/genai/llm_inference/android). TTS («озвучення саммері») — нативний синтез (Android `TextToSpeech` / macOS `AVSpeechSynthesizer`), не LLM. Агент-loop — десктоп; Android стартує з task-specific пайплайну (summarize / classify / propose).

---

## ADR Міграція тестового стеку з bun test на Vitest

## Context and Problem Statement

Монорепо використовувало `bun test` з кастомним `happy-dom.preload.js` (ручна реєстрація DOM + глобалів). `@nitra/cursor` v11 передбачає Vitest як канон (`n-test.mdc` / `n-vue.mdc`); щоразу після Stop-хука регенерувалися `vitest.config.mjs`-файли, несумісні з bun-test сетапом, і ламали лінт.

## Considered Options

* Залишити bun test та ізолювати sync-колатераль у `.gitignore`
* Мігрувати весь тестовий стек на Vitest

## Decision Outcome

Chosen option: "Мігрувати на Vitest", because користувач явно вирішив перейти на Vitest; це також усуває корінь churn — v11-конфіги стають коректними й idempotent, Stop-хук більше не перегенеровує колізійні файли.

### Consequences

* Good, because transcript фіксує очікувану користь: `app/vitest.config.mjs` через `mergeConfig(vite.config.js, …)` дає нативну SFC-компіляцію, авто-імпорти та Quasar без преload-хаків; `vi.resetModules()` замінив `?initial-state`-трюк; `@stryker-mutator/vitest-runner` замінив `command`-runner у `app/stryker.config.mjs`; 85/85 тестів проходять.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Видалено: `app/test/happy-dom.preload.js`, залежності `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc`. Додано до root: `vitest`, `@vitest/coverage-v8`, `@stryker-mutator/vitest-runner`; до `app`: `happy-dom`; до `scripts`: `vitest`. 5 тест-файлів конвертовано: `bun:test` → `vitest` (імпорти, `vi.hoisted` + `vi.mock`, `vi.fn`). `knip.json` — оголошено workspace `scripts`; `.jscpd.json` — ignore `docs/superpowers/**` і `.cursor/**`.
