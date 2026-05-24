---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T21:37:20+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Єдиний оркестратор `scripts/coverage.js` для покриття і мутацій

## Context and Problem Statement

Проєкт має два набори тестів (JS через `bun test`, Rust через `cargo test`) і, після додання мутаційного тестування, ще два окремих інструменти (`bunx stryker run`, `cargo mutants`). Чотири команди потребувалось об'єднати в одну, щоб отримати спільний звіт, придатний для порівняння в git.

## Considered Options

- Два окремих скрипти: `scripts/coverage.js` і `scripts/mutation.js`, кожен оновлює свою секцію в `COVERAGE.md`
- Один скрипт `scripts/coverage.js`, що послідовно запускає всі чотири прогони й перезаписує `COVERAGE.md` цілком

## Decision Outcome

Chosen option: "Один скрипт `scripts/coverage.js`", because користувач явно сказав «я не буду окремо запускати scripts/coverage.js тому поєднуй все в 1 скрипт» — один прогін, одна команда `bun run coverage` у кореневому `package.json`.

### Consequences

- Good, because transcript фіксує очікувану користь: одна точка входу, відсутність синхронізації між двома скриптами.
- Bad, because прогін повільний (Stryker ~8 хв + cargo-mutants ~3 хв + покриття) — transcript підтверджує, але не фіксує це як проблему.

## More Information

- `scripts/coverage.js` — реалізований єдиний `main`: JS lcov → Rust `cargo llvm-cov --json` → Stryker → `cargo mutants --jobs 1`
- `package.json` (корінь): `"coverage": "bun scripts/with-lock.js bun scripts/coverage.js"`

---

## ADR Формат `COVERAGE.md` — одна таблиця без таймстампа

## Context and Problem Statement

Попередня реалізація (від паралельної сесії) писала `# Coverage report — 2026-05-22 04:39:43` і дві окремих таблиці (JS і Rust з різними колонками). Таймстамп засмічував `git diff` при кожному запуску, а дві різні таблиці не давали спільної статистики.

## Considered Options

- Дві секції `## Покриття коду` / `## Мутаційне тестування`, кожна оновлюється своєю командою
- Одна плоска таблиця `| Область | Рядки | Функції | Вбито мутацій | Score |` без таймстампа

## Decision Outcome

Chosen option: "Одна плоска таблиця без таймстампа", because користувач явно запросив «поєднаємо в одну довгу таблицю» та підтвердив мету «щоб можна було порівнювати в гіті» — таймстамп порушував цю мету.

### Consequences

- Good, because transcript фіксує очікувану користь: `git diff` рухається лише коли реально змінюється покриття.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- `COVERAGE.md` у корені репо: `| Область | Рядки | Функції | Вбито мутацій | Score |`
- Рядки: `JS (app)`, `Rust (src-tauri)`, `**Разом**` (зважений підсумок)
- Абсолютні `покрито/всього` збережено — видно приріст коду, не лише відсоток

---

## ADR StrykerJS із command-runner та `inPlace: true` для Bun-монорепо

## Context and Problem Statement

Проєкт використовує `bun test` з власним preload-скриптом замість Vitest (Vitest прибрали раніше відповідно до правила `n-vue.mdc`). StrykerJS не має нативного Bun-раннера; hoisted `node_modules` у Bun-монорепо ламають пісочницю Stryker за замовчуванням.

## Considered Options

- `testRunner: 'vitest'` (vitest-runner) — вимагає Vitest, якого немає в проєкті
- `testRunner: 'command'` + `inPlace: true` — Stryker мутує файли на місці й запускає `bun test` як зовнішню команду

## Decision Outcome

Chosen option: "`testRunner: 'command'` + `inPlace: true`", because Vitest відсутній, а command-runner дозволяє запускати `bun test` без зміни тестового стека. `inPlace` обраний для уникнення проблем зі hoisted-`node_modules` Bun (зафіксовано в коментарі `app/stryker.config.mjs`).

### Consequences

- Good, because transcript фіксує: Stryker успішно завершив прогін (68 killed, 38 survived, 0 errors) коли запускався в ізольованому процесі без гонки.
- Bad, because `inPlace` лишає файли інструментованими при force-kill — transcript зафіксував цей ефект двічі (4 файли відновлювались через `git restore`).

## More Information

- `app/stryker.config.mjs`: `testRunner: 'command'`, `commandRunner.command: 'bun test --preload ./test/happy-dom.preload.js src'`, `inPlace: true`
- `mutate: src/**/*.{js,vue}` (Stryker мутує `<script>`-блоки `.vue`)
- `app/package.json`: `"test:mutation": "bun ../scripts/with-lock.js bunx stryker run"`

---

## ADR `scripts/with-lock.js` — обгортка з PID-lock для захисту від паралельних прогонів

## Context and Problem Statement

Декілька активних Claude-сесій запускали `cargo mutants` і `bunx stryker run` одночасно. Паралельний Stryker із `inPlace` спричинив подвійну інструментацію → `Maximum call stack size exceeded`. Паралельний `cargo-mutants` конфліктував на `mutants.out/`. Lock всередині `coverage.js` захищав лише від другого `coverage.js`, але не від прямих викликів `test:rust:mutation` / `test:mutation`.

## Considered Options

- Lock всередині `coverage.js` — не захищає від запуску підкоманд напряму
- `scripts/with-lock.js` — окремий враппер, через який проходять усі три команди в `package.json`
- `proper-lockfile` npm-пакет — ОС-рівневий lock без stale-проблем, але потребує залежності

## Decision Outcome

Chosen option: "`scripts/with-lock.js`", because це єдиний спосіб закрити всі точки входу (`coverage`, `test:mutation`, `test:rust:mutation`) без нових залежностей. `proper-lockfile` відхилено — transcript фіксує: «ручний PID+liveness достатній і без залежностей».

### Consequences

- Good, because transcript фіксує підтверджений тест: друга команда під час першого прогону виходить з `exit=1` та повідомленням «Метрики вже виконуються»; лок знімається чисто, третя команда виконується.
- Bad, because transcript фіксує обмеження: пряма команда `cargo mutants` у шелі (поза `bun run`) обходить будь-який userspace-lock.

## More Information

- `scripts/with-lock.js`: `.coverage.lock` зі своїм PID; `process.kill(pid, 0)` для liveness-перевірки; реентрантність через `MLMAIL_METRICS_LOCK=1`; `finally` + `SIGINT`/`SIGTERM` хендлери
- `package.json` (корінь): `"coverage": "bun scripts/with-lock.js bun scripts/coverage.js"`
- `app/package.json`: `"test:mutation": "bun ../scripts/with-lock.js bunx stryker run"`, `"test:rust:mutation": "bun ../scripts/with-lock.js cargo mutants --jobs 1 --manifest-path src-tauri/Cargo.toml"`
- `.gitignore` += `.coverage.lock`
