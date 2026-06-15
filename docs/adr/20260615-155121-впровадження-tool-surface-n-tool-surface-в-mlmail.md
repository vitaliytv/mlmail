---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T15:51:21+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

Коміт `967482f` на `main`, лінт exit 0, дерево чисте.

**Що потрібно для запуску логіну:** залишилось заповнити тільки `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID` у `app/src-tauri/.env` — секрет тепер вшито і підхоплюється автоматично. `bun run start` після цього ребілдиться з новим кодом.
---

Цей транскрипт охоплює три незалежних архітектурних рішення.

## ADR Впровадження Tool Surface (n-tool-surface) в mlmail

## Context and Problem Statement

Бачення mlmail передбачає on-device LLM-агент, якому потрібен уніфікований шар для виклику дій застосунку. Без спільного каталогу тулів UI та LLM мали б окремі несинхронізовані шляхи виклику Tauri-команд, що ускладнює підтримку і LLM-інтеграцію.

## Considered Options

* Впровадити n-tool-surface (PortFrom nitra/task): єдиний каталог (`catalog.js`) + dispatch + OpenAI-function-calling manifest; UI і LLM — рівноправні адаптери
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Впровадити n-tool-surface", because це відповідає канону правила `n-tool-surface.mdc` і є передумовою підключення on-device LLM-агента з рівноправним доступом до всіх дій застосунку.

### Consequences

* Good, because `auth-store.js` переведено з прямих `invoke` на `dispatch`; 85/85 JS-тестів продовжили проходити.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файли: `app/src/tool/catalog.js`, `dispatch.js`, `manifest.js`, `transports.js`, `index.js`, `tool.test.js`. Sync правила: `npx @nitra/cursor` → `n-tool-surface.mdc`. ADR: `docs/adr/n-tool-surface-llm-доступний-бекенд.md`. Коміт: `94b77df` (squash до `16f6e7a` Stop-хуком).

---

## ADR Міграція тестового стеку з bun test на Vitest

## Context and Problem Statement

`@nitra/cursor` v11 генерує `vitest.config.mjs` і `@stryker-mutator/vitest-runner` у кожному JS-workspace. Оскільки mlmail використовував `bun test`, ці артефакти конфліктували: knip вважав їх unused/unlisted, а Stop-хук щоходу їх регенерував — повне `bun run lint` не досягало exit 0.

## Considered Options

* Мігрувати на Vitest (узгодитись з v11-канонічним стеком)
* Залишити bun test і розширити knip-ignore на vitest-файли як паліатив
* Відкотити `@nitra/cursor` до v1.27.5

## Decision Outcome

Chosen option: "Мігрувати на Vitest", because це усуває корінь проблеми: v11-конфіги стають коректними і використовуються, Stop-хук їх більше не перегенеровує деструктивно.

### Consequences

* Good, because transcript фіксує очікувану користь: `bun run lint` exit 0, 86/86 тестів зелені; vite-плагіни (Vue/VueMacros/AutoImport/Quasar) нативно доступні у тестах без bun-preload хаків.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файли: `app/vitest.config.mjs` (`mergeConfig(vite.config.js)`), `app/stryker.config.mjs` (testRunner: vitest), 5 конвертованих тест-файлів (`bun:test` → `vitest`, `vi.hoisted`+`vi.mock`, `vi.resetModules()`). Видалено: `test/happy-dom.preload.js`, `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc`. Додано в root: `vitest`, `@vitest/coverage-v8`, `@stryker-mutator/vitest-runner`. jscpd-ignore: `docs/superpowers/**`, `.cursor/**`. knip: оголошено workspace `scripts`. Коміт: `94b77df`.

---

## ADR Вбудований Google OAuth Desktop client_secret як дефолтне значення

## Context and Problem Statement

Desktop OAuth client_secret необхідний для token-exchange, але `app/src-tauri/.env.secret` був відсутній, а `.env` містив порожні значення. Без credentials логін падав з плутаним `AuthError::OAuth` замість пояснення щодо відсутнього конфігу. Виникло питання: зберігати secret поза кодом або вшити в бінарник.

## Considered Options

* Вшити `DESKTOP_SECRET_BUILTIN` безпосередньо в `config.rs` як дефолт, env/`.env.secret` — пріоритетний override
* Тримати лише в `.env.secret` (попередній підхід)

## Decision Outcome

Chosen option: "Вшити `DESKTOP_SECRET_BUILTIN` в `config.rs`", because Google прямо зазначає, що Desktop-client secret є non-confidential (потрапляє в кожен бінарник, який роздається користувачам); захищає флоу PKCE, а не сам secret. Тому різниця в захищеності між env-файлом і вшитим значенням відсутня, але вшите значення не потребує ручного налаштування `.env.secret` на кожній машині розробника.

### Consequences

* Good, because transcript фіксує очікувану користь: `bun run lint` exit 0 після додавання `app/src-tauri/src/auth/config\.rs$` у `.trufflehog-exclude`; cargo build ✓; trufflehog 0 знахідок.
* Bad, because secret закоммічений у git-історію назавжди — ротація (Google Cloud Console) після витоку або компрометації чату є доречною; також secret видимий в чаті сесії.

## More Information

Файли: `app/src-tauri/src/auth/config.rs` (константа `DESKTOP_SECRET_BUILTIN`, `// cspell:disable-line`; функція `desktop_client_secret()` — env → built-in fallback), `.trufflehog-exclude` (додано патерн `app/src-tauri/src/auth/config\.rs$`). Коміт: `967482f`. Для активації логіну потрібен ще `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID` у `.env` — лише secret вшито, client_id — ні.
