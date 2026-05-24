---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T20:10:37+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового середовища з jsdom+Vitest на happy-dom+bun:test

## Context and Problem Statement

Правило `n-vue.mdc` (версія 1.9) забороняє використовувати `jsdom` та `vitest` у воркспейсі `app`. Запуск `npx @nitra/cursor check` повертав `❌` на правилі `vue` через присутність цих залежностей.

## Considered Options

- Залишити jsdom + Vitest (порушення правила)
- Замінити на `@happy-dom/global-registrator` + `bun:test`

## Decision Outcome

Chosen option: "Замінити на `@happy-dom/global-registrator` + `bun:test`", because правило `n-vue.mdc` явно забороняє jsdom і vitest, а `bun:test` є усталеним раннером монорепо.

### Consequences

- Good, because `npx @nitra/cursor check` перейшов із 11/12 до 12/12 — правило `vue` більше не падає.
- Bad, because `bun test` не компілює `.vue` SFC нативно, тому знадобився окремий Bun-плагін у preload (описано в окремому ADR).

## More Information

Змінені файли: `app/package.json` (видалено `jsdom`, `vitest`; додано `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`), `app/vite.config.js` (видалено блок `test`), `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js`. Команда тестів: `bun test --preload ./test/happy-dom.preload.js src`.

---

## ADR Компіляція Vue SFC у bun:test через Bun.plugin + @vue/compiler-sfc

## Context and Problem Statement

Після переходу на `bun:test` виявилося, що Bun завантажує `.vue`-файли як сирі рядки — Vue SFC-компілятор відсутній у рантаймі Bun. Тести компонентів (зокрема `Login.test.js`) падали з `ReferenceError: ref is not defined` та помилками рендеру.

## Considered Options

- Налаштувати Vue SFC-loader (вибір користувача)
- Пропустити компонентні тести (залишити лише юніт)
- Завантажувати `.vue` як рядки

## Decision Outcome

Chosen option: "Налаштувати Vue SFC-loader", because користувач явно обрав цей варіант, коли його запитали.

### Consequences

- Good, because усі 45 тестів проходять (`45 pass / 0 fail`), зокрема компонентні тести `Login.vue` та `App.vue`.
- Good, because transcript фіксує очікувану користь: попередньо зламані тести `auth-store` (падали з `ref is not defined` навіть на `main`) також починають проходити завдяки авто-імпорт-глобалам у preload.
- Bad, because `app/test/happy-dom.preload.js` вимагає ручного перерахування глобалів (`createApp`, `ref`, `computed` тощо) та примусового форсування browser-збірки Quasar через `mock.module('quasar', ...)`.

## More Information

Файл: `app/test/happy-dom.preload.js` — реєструє happy-dom до першого імпорту Vue, компілює `.vue` через `Bun.plugin` + `@vue/compiler-sfc`, робить Vue / Vue Router авто-імпорти глобальними змінними, підміняє `quasar` на `quasar/dist/quasar.client.js`. Файл `app/src/test-utils/quasar.js` реєструє всі Quasar-компоненти глобально (відсутність `@quasar/vite-plugin` під `bun test`).

---

## ADR Формат агрегованого звіту покриття COVERAGE.md

## Context and Problem Statement

Після налаштування JS-покриття (`bun test --coverage`) та Rust-покриття (`cargo llvm-cov`) результати були лише у консолі. Потрібен файл у репозиторії, щоб відстежувати динаміку покриття через git diff.

## Considered Options

- Markdown (COVERAGE.md) — таблиця без таймстампа, лише підсумки
- JSON — машиночитабельний формат
- CSV — табличний формат

## Decision Outcome

Chosen option: "Markdown (COVERAGE.md)", because користувач явно обрав цей варіант; без таймстампа — щоб git diff рухався лише при реальній зміні покриття.

### Consequences

- Good, because git diff одразу показує зміну відсотків; немає шуму від таймстампів у кожному коміті.
- Good, because абсолютні значення `покрито/всього` у форматі `99.74% (390/391)` дозволяють бачити і приріст коду, і відсоток покриття.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Формат таблиці: `| Область | Рядки | Функції |` із рядком `| **Разом** |` (зважений підсумок). Файл: `COVERAGE.md` у корені монорепо. Генерується скриптом `scripts/coverage.js`, запускається командою `bun run coverage` (корінь `package.json`).

---

## ADR Об'єднання всіх метрик (покриття + мутації) в один скрипт

## Context and Problem Statement

Після узгодження дизайну мутаційного тестування (cargo-mutants + StrykerJS) виникло питання: зберігати mutation score в окремому `MUTATION.md` чи в тому ж `COVERAGE.md`, і запускати окремими командами чи однією.

## Considered Options

- Окремий `scripts/mutation.js` + окремий `MUTATION.md`
- Секція «Мутаційне тестування» в `COVERAGE.md`, два скрипти (кожен оновлює свою секцію)
- Усе в один `scripts/coverage.js`, одна команда `bun run coverage`

## Decision Outcome

Chosen option: "Усе в один `scripts/coverage.js`, одна команда `bun run coverage`", because користувач прямо сказав: «я не буду окремо запускати scripts/coverage.js тому поєднуй все в 1 скрипт».

### Consequences

- Good, because один точку входу — `bun run coverage` — замість кількох команд.
- Bad, because команда повільна: мутаційні прогони (`cargo-mutants`, StrykerJS) займають хвилини, тоді як покриття рахується секунди. Запуск одного скрипта завжди запускає обидва slow-кроки.

## More Information

Архітектура скрипта: чотири послідовні кроки — (1) JS-покриття lcov, (2) Rust-покриття `cargo llvm-cov --json`, (3) JS-мутації StrykerJS (`inPlace: true`, command-runner на `bun test`), (4) Rust-мутації `cargo-mutants`. Файл `COVERAGE.md` перезаписується цілком з двома секціями. Стан реалізації на момент зупинки: дизайн узгоджено, паралельна сесія вже частково реалізувала `app/stryker.config.mjs`, `scripts/coverage.js` написано лише під покриття.

---

## ADR Виправлення шляху в .gitignore для артефактів cargo-mutants

## Context and Problem Statement

`cargo-mutants` пише артефакти поруч із `Cargo.toml` — у `app/src-tauri/mutants.out/`. У `.gitignore` був запис `app/mutants.out/` — неправильний шлях, тому директорія `app/src-tauri/mutants.out/` висіла як untracked.

## Considered Options

- `app/mutants.out/` (старий, хибний запис)
- `mutants.out/` (без префікса — матчить на будь-якій глибині)

## Decision Outcome

Chosen option: "`mutants.out/`", because glob без префікса ігнорує `mutants.out/` на будь-якому рівні вкладеності, що покриває як `app/mutants.out/`, так і `app/src-tauri/mutants.out/`.

### Consequences

- Good, because `git check-ignore app/src-tauri/mutants.out` підтверджує ігнорування після зміни.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінений файл: `.gitignore`, рядок 39. Команда верифікації: `git check-ignore app/src-tauri/mutants.out`.
