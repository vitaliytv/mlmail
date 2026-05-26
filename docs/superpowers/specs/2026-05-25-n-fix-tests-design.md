# Design: n-fix-tests — автоматичне посилення тестів на основі mutation coverage

**Дата:** 2026-05-25  
**Репо:** mlmail + @nitra/cursor  
**Статус:** Затверджено, готово до планування

---

## Контекст

`bun run coverage` запускає `n-cursor coverage`, який збирає JS-покриття (lcov) та mutation score (Stryker). Поточна проблема:

1. **Стабільність:** Stryker із 142 мутантами і `concurrency: 1` виконується ~20 хвилин. macOS вбиває процес (SIGURG) до завершення, втрачаючи весь прогрес.
2. **Actionability:** Вижилі мутанти видно лише в `reports/stryker/mutation.json`. COVERAGE.md показує лише агрегований score — без підказок що саме тестувати.
3. **Automation gap:** Немає інструменту, який автоматично дописує тести для вижилих мутантів.

---

## Scope

Три незалежних компоненти, реалізуються послідовно:

### A. Stryker incremental mode (стабільність)

**Зміни в mlmail:**
- `app/stryker.config.mjs`: додати `incremental: true`
- `app/.gitignore` або `.gitignore`: додати `reports/stryker/incremental.json`

**Зміни в @nitra/cursor (для нових проектів):**
- Шаблон stryker.config.mjs, що генерується `n-cursor fix` → `npm_module.stryker_config` — включити `incremental: true` у базовий шаблон

**Поведінка після fix:**
- Якщо SIGURG вбиває Stryker після 100/142 мутантів → наступний `bun run coverage` re-тестує лише 42 survivors/untested, ~3-5 хв замість 20.
- Killed мутанти залишаються killed назавжди (поки вихідний код не змінюється).

---

### B. Recommendations секція в COVERAGE.md

**Мета:** structured markdown-секція, оптимізована як LLM-промпт для наступного агента.

**Формат:**

```markdown
## Вижилі мутанти — рекомендації для дописування тестів

<!-- Автогенеровано n-cursor coverage. Передай LLM як контекст. -->

### `src/i18n/auth-errors.js`

**Рядок 19** — `kind == null || kind === undefined`

| Вижив варіант | Тип мутації | Що не покрито |
|---|---|---|
| `kind === null && kind === undefined` | LogicalOperator | `kind=undefined` де `kind!=null` |
| `false` | ConditionalExpression | умова → true не перевірена |
| `true` | ConditionalExpression | умова → false не перевірена |

### `src/services/auth-store.js`

**Рядок 4** — `authenticated: false`

| Вижив варіант | Тип мутації | Що не покрито |
|---|---|---|
| `true` | BooleanLiteral | початковий стан `authenticated` не перевірено |
```

**Реалізація в @nitra/cursor:**

1. `npm/rules/js-lint/coverage/coverage.mjs` — `parseStrykerReport()`:
   - Додати збір survived мутантів: `{ file, line, original, replacement, type }`
   - Читати вихідний код файлу на `mutant.location.start.line` для поля `original`
   - Повертати `{ lines, functions, killed, total, survived[] }` замість лише `{ lines, functions, killed, total }`

2. `npm/rules/js-lint/coverage/coverage.mjs` — `collect()`:
   - Передавати `survived` у повернуте значення поряд із `coverage` та `mutation`

3. `npm/rules/test/coverage/coverage.mjs` (оркестратор) — `renderMarkdown(rows)`:
   - Якщо будь-який row має `survived` з ненульовою довжиною → append `## Вижилі мутанти` секцію після основної таблиці
   - Групувати по `file`, сортувати по `line`

**Тести:**
- `npm/rules/js-lint/coverage/tests/` — snapshot тест: фіксований `mutation.json` → очікуваний `survived[]`
- `npm/rules/test/coverage/tests/` — snapshot тест: `renderMarkdown` з survived → очікуваний markdown

---

### C. Скіл `.cursor/skills/n-fix-tests/SKILL.md`

**Тригер:** `/n-fix-tests` або `/n-fix-tests 85` (цільовий mutation score %)

**Алгоритм:**

```
1. Знайти mutation.json (шукати в reports/stryker/, app/reports/stryker/)
2. Якщо не знайдено → запропонувати спочатку запустити `bun run coverage`
3. Зчитати survived мутанти, згрупувати по файлах
4. Визначити target_score:
     - Якщо аргумент передано → використати його
     - Інакше → max(current_score + 10, 80)
5. LOOP:
   a. Для кожного файлу з survived мутантами:
      - Прочитати вихідний файл
      - Знайти або створити відповідний тест-файл
      - Дописати тест-кейси, що покривають кожен вижилий мутант
   b. `bun test` → якщо fail → виправити тести (≤ 2 спроби); якщо не виправилось → пропустити мутант із попередженням
   c. `bun run coverage` (incremental → ~3-5 хв)
   d. Прочитати новий score з COVERAGE.md
   e. Якщо score ≥ target_score → DONE (успіх)
   f. Якщо score ≤ попередній score → DONE (конвергенція, звіт про залишкові)
   g. Інакше → оновити survived список, повторити
6. Вивести фінальний звіт: score до/після, кількість вбитих мутантів, залишкові
```

**Де живе:**
- `mlmail/.cursor/skills/n-fix-tests/SKILL.md` — основний скіл
- Реєструється у `mlmail/CLAUDE.md`

**Де знаходити тест-файли:**
- Для `src/services/auth-store.js` → `src/services/auth-store.test.js` або `test/auth-store.test.js`
- Для Vue SFC `src/views/LoginView.vue` → `src/views/__tests__/LoginView.test.js` або наявний файл із `import LoginView`
- Якщо файл не знайдено → створити поруч із джерелом як `<name>.test.js`

**Vue SFC контекст:**
- Тест-файли для Vue потребують `--preload ./test/happy-dom.preload.js` — це вже в `bun test` скрипті
- Агент читає існуючі тест-файли для розуміння конвенцій (imports, setup, mocks)

---

## Порядок реалізації

1. **A** (incremental) — 15 хвилин, одразу дає стабільність
2. **B** (recommendations в COVERAGE.md) — потребує зміни @nitra/cursor + нова версія
3. **C** (скіл n-fix-tests) — після B, бо читає mutation.json (може читати незалежно від B)

Компоненти B і C технічно незалежні — C може бути реалізований паралельно з B, читаючи `mutation.json` напряму.

---

## Out of scope

- CLI команда `n-cursor fix-tests` — MVP через скіл
- Підтримка Rust мутантів (cargo-mutants використовує інший формат)
- Інтеграція з CI (GitHub Actions) — окремий follow-up
