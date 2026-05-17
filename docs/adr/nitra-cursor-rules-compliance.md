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
