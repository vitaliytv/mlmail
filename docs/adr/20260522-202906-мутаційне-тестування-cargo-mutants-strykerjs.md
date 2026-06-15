# Мутаційне тестування: cargo-mutants (Rust) + StrykerJS (JS + Vue)

**Status:** Accepted
**Date:** 2026-05-22

## Context and Problem Statement

Кількісні метрики покриття (`bun test --coverage`, `cargo llvm-cov`) вимірюють лише чи код виконувався, але не чи тести справді перевіряють поведінку. Потрібна якісна метрика, що відповідає на питання «чи тест впаде при поломці коду?».

## Considered Options

- Mutation testing: `cargo-mutants` для Rust, `StrykerJS` для JS/Vue
- Assertion-density аналіз (підрахунок `expect()` на рядок коду)
- Ручний аналіз гілок
- Інші варіанти в transcript не обговорювалися в деталях.

## Decision Outcome

Chosen option: "Mutation testing: `cargo-mutants` для Rust, `StrykerJS` для JS/Vue", because мутаційне тестування — єдина метрика, яка відповідає на питання «чи тест впаде при поломці коду?».

### Consequences

- Good, because `cargo-mutants` виявив 31 незловлену мутацію зі 80 загальних по `src-tauri` — mutation score 46% перед виправленням.
- Good, because StrykerJS покриває `auth-store.js`, `auth-errors.js` і `<script setup>` у `.vue` SFC.
- Bad, because StrykerJS не має нативного runner для `bun test` — використовується command-runner з `inPlace: true`; прогін повільний (хвилини проти секунд для звичайного тесту).
- Neutral, because `bun run coverage` (корінь) запускає єдиний `scripts/coverage.js` що агрегує і покриття, і mutation score в одну таблицю `COVERAGE.md` (`| Область | Рядки | Функції | Вбито мутацій | Score |`) — один повільний прогін замість двох окремих команд (рішення прийнято за прямою вказівкою «поєднуй все в 1 скрипт»).

## More Information

- `app/stryker.config.mjs`: `testRunner: 'command'`, `commandRunner: { command: 'bun test --preload ./test/happy-dom.preload.js src' }`, `inPlace: true`, `mutate: ['src/**/*.{js,vue}', '!src/main.js', '!src/**/*.test.js', '!src/test-utils/**']`
- `app/package.json` скрипти: `"test:mutation": "bunx stryker run"`, `"test:rust:mutation": "cargo mutants --manifest-path src-tauri/Cargo.toml"`
- `.gitignore`: `mutants.out/` (глобус без префікса), `app/reports/`
- Команда верифікації gitignore: `git check-ignore app/src-tauri/mutants.out`
- Додаткова інформація про StrykerJS + Vue SFC: https://stryker-mutator.io/docs/stryker-js/guides/vuejs/
