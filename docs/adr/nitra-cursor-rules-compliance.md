# Приведення проєкту до правил @nitra/cursor через /n-fix

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

Проєкт mlmail накопичив порушення правил `.cursor/rules/` — `npx @nitra/cursor check` повертав `Exit code 1` з кількома `❌`. Запустили скіл `/n-fix` для автоматичного виправлення всіх структурних проблем за один прогін.

## Рішення/Процедура/Факт

- `package.json`: додано скрипти `lint-ga` (`bunx zizmor`), `lint-image` (`npx @nitra/minify-image`); скрипт `lint` розширено `bun run lint-ga` і `oxfmt .`.
- `.markdownlint-cli2.jsonc`: додано `"MD047": false`.
- `.vscode/settings.json`: додано `"[github-actions-workflow]"` з форматтером `vscode-yaml`.
- `.github/workflows/lint-js.yml`: додано `bunx knip` до CI-кроку.
- Створено workflows: `clean-ga-workflows.yml`, `clean-merged-branch.yml`, `lint-ga.yml`, `git-ai.yml`.
- Створено `.github/zizmor.yml` (конфіг аудиту безпеки GitHub Actions).
- `auth-store.js` і `Login.vue`: видалено явний імпорт Vue composables (`readonly`, `ref`, `onMounted`) — надходять через `unplugin-auto-import`.
- Результат: `✨ 10/10 правил без зауважень`.

## Обґрунтування

`n-ga.mdc` вимагає чотирьох workflows і `zizmor.yml`; `n-js-lint.mdc` — `bunx knip` у CI; `n-vue.mdc` — не дублювати auto-imported символи; `n-bun.mdc` — lint-скрипти через `bunx`/`bun run`.

## Розглянуті альтернативи

Не обговорювалися — скіл виконує детерміновані виправлення згідно з правилами.

## Зачіпає

`package.json`, `.markdownlint-cli2.jsonc`, `.vscode/settings.json`, `.github/workflows/lint-js.yml`, `.github/workflows/clean-ga-workflows.yml`, `.github/workflows/clean-merged-branch.yml`, `.github/workflows/lint-ga.yml`, `.github/workflows/git-ai.yml`, `.github/zizmor.yml`, `app/src/services/auth-store.js`, `app/src/views/Login.vue`.

## Update 2026-05-17

Чотири compliance-рішення з однієї сесії `npx @nitra/cursor check` (пакет `@nitra/cursor@1.13.13`):

### Gitleaks як обов'язковий security-лінтер (n-security.mdc)
`npx @nitra/cursor check` повертав три помилки: відсутній `.gitleaks.toml`, відсутній `scripts.lint-security`, `lint-security` не інтегровано в агрегований `lint`. Рішення: створити `.gitleaks.toml` (`[extend] useDefault = true`, `[allowlist]` ігнорує `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures`) і додати `scripts.lint-security = "gitleaks detect --no-banner"` до `package.json` з викликом у агрегованому `lint`. Результат: `12/12 правил без зауважень`.

### Виключення `.claude/worktrees/` з `.gitignore` (jscpd)
`bun run lint` падав на кроці `jscpd` через виявлення дублікатів між робочими файлами та `.claude/worktrees/hopeful-cori-9cef9b/`. `jscpd` поважає `.gitignore` (`gitignore: true` в конфігурації). Рішення: додати `.claude/worktrees/` до `.gitignore`.

### Виключення `.cursor/rules/**` з markdownlint
`bun run lint-text` перевіряв файли в `.cursor/rules/`, які перезаписуються при синхронізації `@nitra/cursor`. Рішення: додати `".cursor/rules/**"` до масиву `ignores` у `.markdownlint-cli2.jsonc`.

### Додавання файлів без Schema Store до `.v8rignore`
`.cursor/hooks.json` та `.gitleaks.toml` не мають схем у Schema Store — `v8r` завершував помилкою `Could not find a schema to validate`. Рішення: додати обидва файли до `.v8rignore` відповідно до `n-text.mdc`.

---

**Опрацьовано** 2026-05-20. Проекції:
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
