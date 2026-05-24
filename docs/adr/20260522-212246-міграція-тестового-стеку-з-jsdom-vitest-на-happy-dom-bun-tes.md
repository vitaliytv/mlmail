---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T21:22:46+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового стеку з jsdom/vitest на happy-dom/bun:test

## Context and Problem Statement

Правило `vue` у `.cursor/rules/n-vue.mdc` забороняє використання `jsdom` і `vitest` у воркспейсі `app`. Проєкт використовував Vitest із jsdom, що призводило до `❌ vue` при перевірці `npx @nitra/cursor check`.

## Considered Options

- Залишити Vitest, замінити jsdom на happy-dom (Vitest підтримує happy-dom через `environment: 'happy-dom'`)
- Замінити повністю на `bun test` із `@happy-dom/global-registrator` і видалити Vitest

## Decision Outcome

Chosen option: "Замінити повністю на `bun test`", because `bun test` — нативний для проєкту (бун-монорепо), правило забороняє `vitest` як залежність незалежно від середовища, а `bun:test` API (`mock`, `mock.module`) забезпечує еквівалентну функціональність.

### Consequences

- Good, because `npx @nitra/cursor check` → 12/12 правил без зауважень (було 11/12).
- Good, because transcript фіксує очікувану користь: бун-плагін у `test/happy-dom.preload.js` автоматично компілює `.vue` SFC і реєструє авто-імпорти Vue/Vue Router.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінені файли: `app/package.json` (прибрано `jsdom`, `vitest`; додано `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`), `app/vite.config.js` (видалено блок `test`), `app/test/happy-dom.preload.js` (новий SFC-плагін для bun), `app/src/test-utils/quasar.js`, `app/src/views/Login.test.js`.

---

## ADR Формат COVERAGE.md без таймстампа

## Context and Problem Statement

Потрібно зберігати звіт покриття в git-trackable форматі, щоб у `git diff` було видно реальні зміни відсотків між комітами. Паралельна сесія згенерувала `COVERAGE.md` із заголовком `# Coverage report — 2026-05-22 04:39:43`, що засмічує diff при кожному прогоні.

## Considered Options

- Markdown із таймстампом (як генерувала паралельна сесія)
- Markdown без таймстампа (лише підсумкові рядки таблиці)
- JSON або lcov-формат

## Decision Outcome

Chosen option: "Markdown без таймстампа", because `git diff` тоді показує зміни лише тоді, коли справді змінюється покриття, без зайвого шуму від дати.

### Consequences

- Good, because transcript фіксує очікувану користь: «diff рухається тільки коли реально змінилось покриття».
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файл: `COVERAGE.md` у корені монорепо. Генерується скриптом `scripts/coverage.js`. Запуск: `bun run coverage`.

---

## ADR Єдиний скрипт-оркестратор для покриття й мутацій

## Context and Problem Statement

Проєкт має два виміри метрик якості тестів: кількісне покриття (JS — lcov, Rust — cargo-llvm-cov) і мутаційне тестування (JS — StrykerJS, Rust — cargo-mutants). Паралельна сесія реалізувала два окремі скрипти (`coverage.js` + `mutation.js`) із двома секціями в `COVERAGE.md`. Користувач вимагав «поєднати в один скрипт, один прогін, одна таблиця».

## Considered Options

- Два окремі скрипти та дві секції у `COVERAGE.md` (кожну оновлює своя команда)
- Один скрипт `scripts/coverage.js`, один прогін: coverage JS → coverage Rust → mutation JS → mutation Rust → одна таблиця
- Склейка команд через `&&` у `package.json`

## Decision Outcome

Chosen option: "Один скрипт, один прогін, одна таблиця", because користувач явно вказав: «я не буду окремо запускати scripts/coverage.js — поєднуй все в 1 скрипт».

### Consequences

- Good, because один виклик `bun run coverage` дає повну картину якості в одному `COVERAGE.md`.
- Bad, because прогін повільний (~15 хв): Stryker ~8 хв + cargo-mutants ~3 хв + coverage секунди. Transcript фіксує це як відоме обмеження.

## More Information

Скрипт: `scripts/coverage.js`. Root `package.json`: `"coverage": "bun scripts/with-lock.js bun scripts/coverage.js"`. Таблиця: `| Область | Рядки | Функції | Вбито мутацій | Score |`. Парсинг: JS через lcov (`LF/LH/FNF/FNH`), Rust через `cargo llvm-cov --json --summary-only`, JS-мутації через `reports/stryker/mutation.json`, Rust-мутації через `mutants.out/outcomes.json`.

---

## ADR PID-liveness lock для захисту від паралельних прогонів

## Context and Problem Statement

Паралельна робота двох агентських сесій над одним репозиторієм призвела до одночасного запуску кількох прогонів `cargo-mutants` і Stryker. StrykerJS із `inPlace: true` при force-kill залишив мутовані вихідні файли (`auth-store.js`, `auth-errors.js`, `App.vue`, `Login.vue`) у інструментованому стані. Два паралельні Stryker-прогони спричинили подвійну інструментацію → `Maximum call stack size exceeded`.

## Considered Options

- PID + liveness lock: lock-файл `.coverage.lock` зі своїм PID; перевірка живучості через `process.kill(pid, 0)` (обраний варіант)
- OS-level lock (`proper-lockfile`): ОС автоматично звільняє при смерті процесу — найнадійніше, але вимагає залежності
- Скан процесів через `pgrep`: без файлів, але є вікно гонки TOCTOU і крихкість по іменах

## Decision Outcome

Chosen option: "PID + liveness lock", because не вимагає нових npm-залежностей, а stale-лок обробляється через liveness-перевірку PID.

### Consequences

- Good, because transcript фіксує очікувану користь: одночасний `bun run coverage` від другого процесу отримує «вже виконується» і завершується без запуску.
- Good, because реєнтрантність через env-змінну `MLMAIL_METRICS_LOCK=1` запобігає дедлоку, коли `coverage.js` усередині викликає підкоманди, обгорнуті в `with-lock.js`.
- Bad, because прямий `cargo mutants` із шелу обходить lock — transcript явно фіксує це обмеження як прийняте («навмисна ручна дія, не агентська гонка»).

## More Information

Файли: `scripts/with-lock.js` (новий), `.coverage.lock` (у `.gitignore`). Обгортка застосована у всіх точках входу: `coverage` (корінь), `test:mutation` і `test:rust:mutation` (`app/package.json`). Відповідний запис у `.gitignore`: `# coverage.js single-run lock` → `.coverage.lock`.

---

## ADR Ігнорування exit code cargo-mutants при парсингу

## Context and Problem Statement

`cargo-mutants` виходить із кодом 1 коли є missed або timeout мутанти — це штатна поведінка (не помилка інструменту), а звіт про непокриті мутанти. Перша реалізація `rustMutation()` перевіряла `if (exitCode !== 0) { return null }`, що призводило до `—` у колонках Rust-мутацій у `COVERAGE.md` навіть при успішному прогоні з `outcomes.json` на диску.

## Considered Options

- Перевіряти exit code і повертати `null` при ненульовому (перша реалізація)
- Ігнорувати exit code, завжди читати `mutants.out/outcomes.json` (обраний варіант)

## Decision Outcome

Chosen option: "Ігнорувати exit code, читати outcomes.json", because ненульовий exit — частина нормального звіту cargo-mutants, а не ознака збою; `outcomes.json` містить валідні дані незалежно від результату мутацій.

### Consequences

- Good, because transcript фіксує очікувану користь: Rust mutation score відображається коректно (37/68 caught, ~54.4%) навіть коли є missed/timeout мутанти.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Функція `rustMutation()` у `scripts/coverage.js`: викликає `run('cargo', ['mutants', ...])` без перевірки `exitCode`, потім читає `mutants.out/outcomes.json`. Поля: `caught` → killed, `caught + missed` → total (timeout-мутанти не включаються в знаменник).
