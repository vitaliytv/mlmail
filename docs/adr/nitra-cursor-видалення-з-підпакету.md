# Видалення `@nitra/cursor` з `app/package.json`

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

`@nitra/cursor` потрапив до `devDependencies` у `app/package.json` — коли `npx @nitra/cursor` запускався з підкаталогу `app/`, пакет записувався в найближчий `package.json`. Lint-інструмент (`knip`) щоразу повідомляв про `Unused devDependencies: @nitra/cursor` у workspace `app`, бо пакет не імпортується жодним модулем Vite/Tauri-застосунку.

## Considered Options

- Видалити `@nitra/cursor` з `app/package.json`, залишити в кореневому `package.json`.
- Виключити `@nitra/cursor` з конфігурації `knip` у workspace `app`.
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Видалити з app/package.json", because пакет використовується лише кореневими скриптами (`lint-ga`, `lint-text` тощо) і не потрібен у workspace `app`. Видалення усуває першопричину lint-попередження замість його маскування.

### Consequences

- Good, because lint (`knip`) більше не скаржиться на невикористану залежність у `app/`.
- Good, because монорепо-структура відповідає принципу: CLI-інструменти — у кореневому workspace.
- Neutral, because `bun install` синхронізує `bun.lock` після видалення.
- Bad, because transcript не містить підтвердження про потенційні наслідки для інших workspace-ів.

## More Information

Зачіпає: `app/package.json` (видалено рядок `@nitra/cursor` з `devDependencies`), `bun.lock` (оновлено після `bun install`). У кореневому `package.json` залежність збережена.

---

**Опрацьовано** 2026-05-20. Проекції:

- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
