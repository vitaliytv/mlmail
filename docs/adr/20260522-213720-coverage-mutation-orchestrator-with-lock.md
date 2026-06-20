# Єдиний оркестратор `scripts/coverage.js` з PID-lock захистом

**Status:** Accepted
**Date:** 2026-05-22

## Context and Problem Statement

Проєкт має два набори тестів (JS і Rust) та два мутаційних інструменти (StrykerJS і `cargo-mutants`). Чотири команди потребувалось об'єднати в одну для спільного git-trackable звіту `COVERAGE.md`. Паралельна агентська робота призводила до одночасного запуску Stryker/cargo-mutants: StrykerJS з `inPlace: true` при force-kill залишав файли в інструментованому стані (`stryNS_…` маркери); паралельний Stryker спричиняв `Maximum call stack size exceeded`.

## Considered Options

- Два окремі скрипти (`scripts/coverage.js` + `scripts/mutation.js`) з двома секціями у `COVERAGE.md`
- Один скрипт `scripts/coverage.js` з одним прогоном і `scripts/with-lock.js` PID-liveness lock
- Склейка команд через `&&` у `package.json` без скрипта-оркестратора

## Decision Outcome

Chosen option: "Один скрипт `scripts/coverage.js` з `scripts/with-lock.js`", because користувач явно вказав «я не буду окремо запускати — поєднуй все в 1 скрипт»; PID-liveness lock без нових npm-залежностей закриває всі точки входу від паралельного запуску.

### Consequences

- Good, because один виклик `bun run coverage` дає повну картину якості в єдиному `COVERAGE.md` без таймстампа — `git diff` рухається лише при реальній зміні метрик.
- Good, because підтверджено: другий паралельний виклик отримує «Метрики вже виконуються» і завершується з exit=1 без запуску мутацій; лок знімається чисто, наступний виклик виконується.
- Bad, because прогін повільний (~8 хв Stryker + ~3 хв cargo-mutants + секунди на coverage).
- Bad, because `inPlace: true` при force-kill залишає файли в інструментованому стані — потрібне `git restore` вручну; пряма команда `cargo mutants` з шелу обходить userspace-lock (прийняте обмеження).
- Neutral, because `cargo-mutants` виходить із кодом 1 при missed/timeout мутантах (штатна поведінка) — `coverage.js` ігнорує exit code і завжди читає `mutants.out/outcomes.json`.

## More Information

- `scripts/coverage.js`: послідовно запускає JS lcov → Rust `cargo llvm-cov --json` → Stryker → `cargo mutants --jobs 1`; перезаписує `COVERAGE.md` з таблицею `| Область | Рядки | Функції | Вбито мутацій | Score |` без таймстампа.
- `scripts/with-lock.js`: lock-файл `.coverage.lock` зі своїм PID; `process.kill(pid, 0)` для liveness-перевірки stale-lock; реентрантність через env `MLMAIL_METRICS_LOCK=1`; `finally` + `SIGINT`/`SIGTERM` хендлери.
- `package.json` (корінь): `"coverage": "bun scripts/with-lock.js bun scripts/coverage.js"`.
- `app/package.json`: `"test:mutation": "bun ../scripts/with-lock.js bunx stryker run"`, `"test:rust:mutation": "bun ../scripts/with-lock.js cargo mutants --jobs 1 --manifest-path src-tauri/Cargo.toml"`.
- `app/stryker.config.mjs`: `testRunner: 'command'`, `commandRunner.command: 'bun test --preload ./test/happy-dom.preload.js src'`, `inPlace: true`, `mutate: ["src/**/*.{js,vue}", "!src/**/*.test.js", "!src/test-utils/**", "!src/main.js"]`, `concurrency: 1`, `timeoutMS: 60000`.
- `.gitignore`: `mutants.out/`, `app/reports/`, `.coverage.lock`.
