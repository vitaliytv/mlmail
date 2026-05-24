# Mutation Testing — специфікація

**Дата:** 2026-05-22
**Проєкт:** mlmail (Tauri + Vue 3 + Rust)

---

## Мета

Додати мутаційне тестування (якісна метрика покриття) до наявного кількісного звіту. Показник: mutation score — відсоток мутантів, яких вбили тести. Результат зберігається у `COVERAGE.md` поруч з кількісними рядками, порівнювати можна через `git diff`.

---

## Область дії

**Ціль мутацій:**

- Rust: весь `app/src-tauri/src/` (auth-флоу, Gmail-інтеграція, utils)
- JS: лише файли з реальною логікою:
  - `app/src/services/auth-store.js`
  - `app/src/i18n/auth-errors.js`

**Поза областю:**

- `.vue`-файли: Stryker 8 не мутує SFC, у наявних файлах немає логіки для мутації
- `main.js`, bootstrap-код — не мутується

---

## Інструменти

| Мова | Інструмент              | Версія                                         |
| ---- | ----------------------- | ---------------------------------------------- |
| Rust | `cargo-mutants`         | остання стабільна (глобальний `cargo install`) |
| JS   | `@stryker-mutator/core` | ^8 (devDep у `app/package.json`)               |

---

## Нові файли та зміни

### `app/package.json`

Нові devDep:

```json
"@stryker-mutator/core": "^8.x"
```

Нові скрипти:

```json
"mutation:rust": "cargo mutants --manifest-path src-tauri/Cargo.toml",
"mutation:js":   "stryker run"
```

### `app/stryker.config.mjs` (новий)

```js
export default {
  testRunner: 'command',
  commandRunner: {
    command: 'bun test --preload ./test/happy-dom.preload.js src'
  },
  mutate: ['src/services/auth-store.js', 'src/i18n/auth-errors.js'],
  reporters: ['json'],
  jsonReporter: { fileName: 'reports/stryker/mutation.json' },
  tempDirName: 'reports/stryker/.tmp',
  coverageAnalysis: 'off'
}
```

`coverageAnalysis: 'off'` — bun:test не підтримує Istanbul-інструментацію, яку Stryker очікує для аналізу покриття.

### `scripts/coverage.js`

Нова команда `mutation` (окрема від `coverage`):

1. Запускає `bun --cwd=app run mutation:rust` → парсить stdout рядки виду `MISSED`, `CAUGHT`, `TIMEOUT`, `UNVIABLE`
2. Запускає `bun --cwd=app run mutation:js` → читає `app/reports/stryker/mutation.json`, поля `killed`, `survived`, `noCoverage`, `timeout`
3. Рахує score = killed / (killed + survived + noCoverage + timeout) × 100
4. Записує (або замінює наявну) секцію `## Mutation score` у `COVERAGE.md`:

```markdown
## Mutation score

| Область   | Мутантів | Вбито | Score |
| --------- | -------- | ----- | ----- |
| Rust      | N        | N     | N%    |
| JS        | N        | N     | N%    |
| **Разом** | N        | N     | N%    |
```

**Без таймстампа.** `COVERAGE.md` змінюється лише коли змінюється score.

### Кореневий `package.json`

Новий скрипт:

```json
"mutation": "bun scripts/coverage.js mutation"
```

### `.gitignore`

Додати:

```
app/reports/stryker/.tmp/
```

`app/reports/stryker/mutation.json` **не** ігнорується — комітиться як артефакт для порівняння.

---

## Команди після реалізації

```bash
bun run coverage          # кількісний звіт (рядки/функції)
bun run mutation          # мутаційний звіт → COVERAGE.md
```

---

## Обмеження та застереження

- `cargo-mutants` на повному `src-tauri` може займати 5–15 хв (кожна мутація — перезбирання Rust). Рекомендовано запускати локально перед PR, а не в кожному CI-прогоні.
- Stryker command-runner запускає `bun test` на **кожному** мутанті незалежно → повільніше, ніж з coverage-аналізом, але це єдиний надійний спосіб з bun:test.
- Stryker за замовчуванням не змінює оригінальні файли — мутовані копії ізольовані в `.tmp`-пісочниці.

---

## Changelog

- `app` version bump → `0.1.3`, запис у `app/CHANGELOG.md`
- `scripts` version bump → `0.1.2`, запис у `scripts/CHANGELOG.md`
