# Coverage Fix Agent — Design Spec

**Дата:** 2026-05-25  
**Статус:** Approved

---

## Мета

Розширити `n-cursor coverage` двома незалежними покращеннями:

1. **Stryker incremental mode** — стабільність: якщо процес перерваний (SIGURG, OOM), наступний запуск продовжує з того місця, а не починає з нуля.
2. **`--fix` режим** — автоматично запускає Claude Code агента для написання тестів по вижилих мутантах, потім повторно валідує coverage.

---

## Архітектура

### Компонент 1: Stryker incremental mode

**Де:** `@nitra/cursor` — шаблон `stryker.config.mjs` (генерується командою `n-cursor fix`).

**Зміни:**

```js
export default {
  // ...existing config...
  incremental: true,
  incrementalFile: 'reports/stryker/stryker-incremental.json'
}
```

**`.gitignore` entry:** `reports/stryker/stryker-incremental.json`

Якщо `incrementalFile` існує, Stryker пропускає мутантів зі збереженим результатом. Якщо файл не існує — повний прогін.

---

### Компонент 2: Recommendations у COVERAGE.md

**Де:** `@nitra/cursor` — `rules/js-lint/coverage/coverage.mjs` + `rules/test/coverage/coverage.mjs`.

**`parseStrykerReport` (js-lint provider) — доповнений результат:**

```js
return {
  coverage: { lines, functions },
  mutations: { killed, total },
  survived: [
    {
      file: 'src/i18n/auth-errors.js',
      line: 19,
      original: 'kind === "not-found" || kind == null',
      replacement: 'false',
      type: 'ConditionalExpression'
    }
    // ...
  ]
}
```

Для отримання `original` — читаємо рядок з source file через `fs.readFileSync` за шляхом з `mutation.json` (`file.source` або `jsRoot + mutant.fileName`).

**`renderMarkdown` (orchestrator) — новий розділ:**

```markdown
## Рекомендації (46 вижилих мутантів)

### `src/i18n/auth-errors.js`

| Рядок | Оригінал                                 | Мутант                                | Тип                   |
| ----- | ---------------------------------------- | ------------------------------------- | --------------------- |
| 19    | `kind === 'not-found' \|\| kind == null` | `false`                               | ConditionalExpression |
| 19    | `kind === 'not-found' \|\| kind == null` | `kind === null && kind === undefined` | LogicalOperator       |

### `src/services/auth-store.js`

...
```

Розділ рендериться завжди (не лише при `--fix`), якщо є вижилі мутанти.

---

### Компонент 3: `--fix` режим

**CLI UX:**

```bash
n-cursor coverage        # метрики → COVERAGE.md (як зараз + розділ Рекомендації)
n-cursor coverage --fix  # те саме + агент → coverage знову
```

**Файл:** `npm/scripts/coverage-fix.mjs` (новий)

```js
import { query } from '@anthropic-ai/claude-code'

export async function fixSurvivedMutants(survivedMutants, projectRoot) {
  const prompt = buildFixPrompt(survivedMutants, projectRoot)

  for await (const msg of query({
    prompt,
    options: {
      cwd: projectRoot,
      maxTurns: 20,
      allowedTools: ['Read', 'Edit', 'Bash']
    }
  })) {
    if (msg.type === 'text') process.stdout.write(msg.text)
  }
}

function buildFixPrompt(survived, projectRoot) {
  // Для кожного вижилого мутанта: file, line, original, replacement, type
  // + контекст: ±5 рядків з source file
  // + шлях до відповідного test file (евристика: src/foo.js → test/foo.test.js або src/__tests__/foo.test.js)
  // Інструкція: "напиши тести що вбивають ці мутанти, запусти bun test щоб переконатись"
}
```

**Потік `--fix`:**

1. Запускає `n-cursor coverage` (збирає метрики, пише COVERAGE.md з рекомендаціями)
2. Зчитує `survived` з результату `collect()`
3. Якщо `survived.length === 0` — виводить "Всі мутанти вбиті" і завершує
4. Викликає `fixSurvivedMutants(survived, projectRoot)`
5. Після завершення агента — запускає `n-cursor coverage` знову (без `--fix`)
6. Виводить diff між старим і новим score

---

## Залежності

**`@nitra/cursor` `npm/package.json`:**

```json
"dependencies": {
  "@anthropic-ai/claude-code": "latest"
}
```

**Версія:** 1.20.0 (minor bump — нова функціональність)

---

## Тести

- `rules/js-lint/coverage/tests/` — unit тест: `parseStrykerReport` з фіксованим `mutation.json` повертає правильний `survived` масив
- `rules/test/coverage/tests/` — unit тест: `renderMarkdown` з рядками що мають `survived` рендерить розділ `## Рекомендації`
- Інтеграційний тест `--fix` — мокований `query()` або окремий fixture

---

## Поза скоупом

- Ітеративний цикл (пише тести → coverage → пише ще) — тільки один цикл
- Rust coverage fix — тільки JS/Stryker
- Конфігурація моделі — використовує дефолт Claude Code
