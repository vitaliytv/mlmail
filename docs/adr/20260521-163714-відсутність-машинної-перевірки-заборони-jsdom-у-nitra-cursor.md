---
session: 94e86f54-44e9-4c1f-a123-8f3a17ef33a3
captured: 2026-05-21T16:37:14+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/94e86f54-44e9-4c1f-a123-8f3a17ef33a3.jsonl
---

## ADR Відсутність машинної перевірки заборони `jsdom` у `@nitra/cursor`

## Context and Problem Statement

Правило `n-vue.mdc` v1.9 (рядок 120) прямо забороняє `jsdom` у Vue-проєктах на користь `happy-dom`. Але `npx @nitra/cursor check` цю заборону не перевіряє — `check.mjs` і `package_json.rego` у `rules/vue/` не містять жодного токену `jsdom`/`vitest`. Через це `app/package.json` з `"jsdom": "^29.1.1"` і `"vitest": "^4.1.7"` не дає `❌` і `n-fix` не реагує. Порушення непомітне без ручного читання правила.

## Considered Options

- Додати машинну перевірку (`jsdom` у `devDependencies` → `❌`) у `rules/vue/policy/package_json/package_json.rego` або `rules/vue/fix/packages/check.mjs`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Підготувати патч для `@nitra/cursor` через `/n-llm-patch`", because користувач явно запустив `/n-llm-patch` з аргументом «патч для @nitra/cursor» після того, як була виявлена відсутня перевірка.

### Consequences

- Good, because transcript фіксує очікувану користь: після патча `npx @nitra/cursor check` видаватиме `❌` при наявності `jsdom`/`vitest` у `devDependencies` Vue-проєкту, і `n-fix` зможе прибирати порушення автоматично.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файли, які потребують зміни у `@nitra/cursor`:

- `rules/vue/policy/package_json/package_json.rego` — додати заборону на `jsdom` (і, ймовірно, `vitest`) у `devDependencies`
- `rules/vue/fix/packages/check.mjs` (версія 1.13.72) — сигналізує лише про `vite-env.d.ts`, `jsconfig.json`, `VueMacros`, `AutoImport`, `esbuild`; `jsdom`/`vitest` не покривається

Поточний стан порушення у репо:

- `app/package.json:29` — `"jsdom": "^29.1.1"` (має бути прибраний)
- `app/package.json:34` — `"vitest": "^4.1.7"` (має бути прибраний)
- `app/package.json:13-14` — скрипти `test:ui` і `test:watch` використовують `vitest` (мають перейти на `bun test`)
- `happy-dom` у `app/package.json` відсутній (має бути доданий)
