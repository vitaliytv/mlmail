---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T19:33:05+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

This transcript documents a significant architectural decision about Vue testing infrastructure. Let me generate the ADR.

## ADR Міграція тестового середовища Vue з jsdom/vitest на happy-dom/bun test

## Context and Problem Statement

Правило `vue` у `.cursor/rules/n-vue.mdc` забороняє використання `jsdom` і `vitest`. Запуск `npx @nitra/cursor check` фіксував один `❌` у правилі `vue`, оскільки `app/package.json` містив обидві залежності. Запуск `/n-fix` мав усунути це порушення.

## Considered Options

- Замінити `jsdom` і `vitest` на `@happy-dom/global-registrator` + `bun test`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Замінити `jsdom` і `vitest` на `@happy-dom/global-registrator` + `bun test`", because проєкт вже використовує Bun як єдиний package manager (правило `n-bun.mdc`), а правило `n-vue.mdc` явно забороняє `jsdom` і `vitest`.

### Consequences

- Good, because `npx @nitra/cursor check` перейшов з 11/12 на 12/12 після міграції.
- Bad, because `bun test` не компілює `.vue` SFC з коробки — це потребувало додаткового `Bun.plugin` на `@vue/compiler-sfc` у файлі `app/test/happy-dom.preload.js`; без нього компонентні тести (`Login.test.js`) падають з `ReferenceError: ref is not defined` або отримують `.vue`-файл як рядок.

## More Information

Змінені файли:

- `app/package.json` — прибрано `jsdom`, `vitest`; додано `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`
- `app/vite.config.js` — прибрано блок `test` (конфіг Vitest)
- `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js` — переписано під `bun:test` (`mock`, `mock.module` замість `vi`)
- `app/test/happy-dom.preload.js` — новий файл; реєструє happy-dom, компілює `.vue` через `Bun.plugin`, форсує browser-збірку Quasar (`quasar/dist/quasar.client.js`), віддає Vue / Vue Router як глобали

Команда тестів: `bun test --preload ./test/happy-dom.preload.js src`
Quasar завантажує серверну збірку (`quasar.server.prod.js`) під умовою `node` у полі `exports`, що спричиняє `Object.assign requires input not be null` — обходиться через `mock.module('quasar', () => require('quasar/dist/quasar.client.js'))` у preload.

---

## ADR Об'єднаний скрипт покриття JS і Rust із генерацією COVERAGE.md

## Context and Problem Statement

У проєкті є два незалежних скрипти покриття: `bun test --coverage` (JS/Vue) і `cargo llvm-cov` (Rust). Результати — у різних форматах, без загальної статистики. Користувач хоче одну команду, яка дає спільний підсумок в одному форматі та зберігає результат у `COVERAGE.md` для порівняння через `git diff`.

## Considered Options

- Скрипт-оркестратор `scripts/coverage.js` — парсить lcov і JSON, рахує зважений підсумок, генерує `COVERAGE.md` без таймстампа
- Склейка з `&&` у `package.json` (відхилено — немає спільної статистики, регекспи крихкі)
- Зліплення двох нативних таблиць у файл (відхилено — не «один формат», немає рядка «Разом»)

## Decision Outcome

Chosen option: "Скрипт-оркестратор `scripts/coverage.js`", because таймстамп у файлі засмічує `git diff` при кожному запуску, тоді як спільний підсумок і відсутність таймстампа дають чистий diff лише при реальних змінах покриття.

### Consequences

- Good, because `git diff COVERAGE.md` показує зміни лише тоді, коли реально змінилось покриття (таймстампа немає); є рядок «Разом» із зваженою статистикою.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Формат `COVERAGE.md`:

```
| Область          | Рядки              | Функції          |
| JS (app)         | 99.74% (390/391)   | 100.00% (50/50)  |
| Rust (src-tauri) | 83.18% (1231/1480) | 71.30% (164/230) |
| **Разом**        | 86.64% (1621/1871) | 76.43% (214/280) |
```

Змінені файли:

- `scripts/coverage.js` — парсить lcov (`LF`/`LH`/`FNF`/`FNH`) з тимчасової теки через `bun test --reporter lcov`; парсить Rust через `cargo llvm-cov --json --summary-only`; пише `COVERAGE.md` у корінь репо
- `package.json` (корінь) — додано `"coverage": "bun scripts/coverage.js"`
- `scripts/package.json` → `0.1.1`, `scripts/CHANGELOG.md` — новий запис
- `COVERAGE.md` — генерований файл, комітується в репозиторій

Команда запуску: `bun run coverage`
