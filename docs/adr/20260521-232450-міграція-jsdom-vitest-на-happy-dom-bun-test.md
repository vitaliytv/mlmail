# Міграція тестового середовища з jsdom + Vitest на happy-dom + bun:test

**Status:** Accepted
**Date:** 2026-05-21

## Context and Problem Statement

Правило `n-vue.mdc` у `.cursor/rules/` явно забороняє `jsdom` і `vitest` у воркспейсі `app`. Команда `npx @nitra/cursor check` повертала `❌` на правилі `vue` через наявність цих залежностей у `app/package.json` та блоку `test` у `app/vite.config.js`.

## Considered Options

- Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun:test`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun:test`", because правило `n-vue.mdc` явно забороняє `jsdom` і `vitest`, а `bun:test` вже є єдиним дозволеним test-runner-ом у монорепо згідно з правилом `n-bun.mdc`.

### Consequences

- Good, because `npx @nitra/cursor check` перейшов із 11/12 на 12/12 — порушення правила `vue` усунено.
- Bad, because `bun test` не компілює `.vue` SFC нативно — компонентні тести потребують окремого `Bun.plugin` (задокументовано в окремому ADR).

## More Information

- `app/package.json`: видалено `jsdom`, `vitest`; додано `@happy-dom/global-registrator ^20.9.0`, `@types/bun ^1.3.14`
- `app/vite.config.js`: видалено блок `test` (конфіг Vitest)
- `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js`: `vi.fn()` / `vi.mock` замінено на `mock()` / `mock.module` з `bun:test`
- Скрипт `test` у `app/package.json`: `bun test --preload ./test/happy-dom.preload.js src`
- Версія `app` підвищена `0.1.0` → `0.1.1`, створено `app/CHANGELOG.md` (правило `n-changelog.mdc`)
