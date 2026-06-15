# Агрегований звіт покриття JS і Rust у COVERAGE.md

**Status:** Accepted
**Date:** 2026-05-22

## Context and Problem Statement

У проєкті є два незалежних інструменти покриття: `bun test --coverage` (JS/Vue, `app/`) і `cargo llvm-cov` (Rust, `app/src-tauri/`). Результати існували лише в консолі в різних форматах, без збереження для відстеження динаміки між запусками.

## Considered Options

- Єдиний скрипт `scripts/coverage.js`, що парсить обидва джерела й записує підсумки у `COVERAGE.md` у корені монорепо
- JSON або LCOV як формат файлу
- Markdown із повними таблицями файлів (включно з іменами файлів)

## Decision Outcome

Chosen option: "Єдиний скрипт із Markdown-підсумками у `COVERAGE.md` без таймстампа", because користувач явно обрав «Markdown (COVERAGE.md)» і «Лише підсумки» через `AskUserQuestion` у transcript; таймстамп у файлі засмічує `git diff` при кожному запуску — його відсутність дає чистий diff лише при реальній зміні покриття.

### Consequences

- Good, because `git diff COVERAGE.md` між двома запусками показує зміну лише при реальних змінах покриття (таймстампа немає).
- Good, because абсолютні значення `покрито/всього` у форматі `99.74% (390/391)` дозволяють бачити і приріст коду, і відсоток покриття.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Формат `COVERAGE.md`:

```
| Область          | Рядки              | Функції          |
| JS (app)         | 99.74% (390/391)   | 100.00% (50/50)  |
| Rust (src-tauri) | 83.18% (1231/1480) | 71.30% (164/230) |
| **Разом**        | 86.64% (1621/1871) | 76.43% (214/280) |
```

- `scripts/coverage.js`: парсить lcov (`LF`/`LH`/`FNF`/`FNH`) з тимчасової директорії через `bun test --reporter lcov`; парсить Rust через `cargo llvm-cov --json --summary-only`; записує `COVERAGE.md` у корінь монорепо без таймстампа
- `package.json` (корінь): `"coverage": "bun scripts/coverage.js"`
- Залежності: `cargo-llvm-cov v0.8.7` + rustup-компонент `llvm-tools-preview`
- `COVERAGE.md` комітується в репозиторій

## Update 2026-05-22

Уточнення реалізації `scripts/coverage.js` (сесія `a8164e48`, 19:33):

- `bun test` запускається з `--reporter lcov` у тимчасову директорію; lcov-файл парситься за полями `LF`/`LH` (рядки) і `FNF`/`FNH` (функції)
- `cargo llvm-cov` викликається з `--json --summary-only`; з відповіді читаються `lines.percent`, `lines.count`, `lines.covered`, `functions.percent`, `functions.count`, `functions.covered`
- Рядок `**Разом**` розраховується як зважена сума абсолютних чисел (не середнє відсотків)
- Таймстамп свідомо відсутній — `git diff` рухається лише при реальній зміні покриття
- `scripts/package.json` підвищено до `0.1.1`; `scripts/CHANGELOG.md` оновлено відповідним записом
