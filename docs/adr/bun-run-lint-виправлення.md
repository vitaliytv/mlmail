# Виправлення lint-ланцюжка bun run lint після n-fix

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

Після /n-fix кілька інструментів у `bun run lint` стали падати: knip знаходив невикористані залежності, v8r намагався валідувати Rust/Tauri-файли, cspell не знав UA-транслітерацій, shellcheck знаходив SC2086 у workflow.

## Рішення/Процедура/Факт

- `app/package.json`: видалено `@tauri-apps/plugin-opener` (JS-сторона не використовувала; Rust-сторона залишилась).
- `knip.json`: у `ignore` — `app/src-tauri/target/**`; у `ignoreDependencies` — `@nitra/stylelint-config`; у `ignoreBinaries` — `stylelint`.
- `.cspell.json`: виправлено одруківку `rендер` (латинська `r`) → `рендер`; додано 13 UA-транслітерацій (`фічі`, `хелпери`, `стейтів`, `скіл` тощо).
- `.v8rignore`: переписано у gitignore-style з trailing-slash; додано `app/src-tauri/target/`, `app/src-tauri/gen/`, `app/src-tauri/capabilities/`, `app/src-tauri/tauri.conf.json`.
- `.github/workflows/git-ai.yml`: `$GITHUB_PATH` → `"$GITHUB_PATH"` (SC2086).
- Результат: `bun run lint` exit 0.

## Обґрунтування

Всі виправлення реальні: невикористана залежність видалена, одруківку виправлено в джерелі, Tauri-генеровані файли виключені з v8r (їх схеми Schema Store не знає), UA-транслітерації потрібні в ADR-документах.

## Розглянуті альтернативи

`@tauri-apps/plugin-opener` можна було перенести в `ignoreDependencies`, але JS-код на нього не посилається — видалення правильніше. Для v8r розглядалися `**`-префіксні патерни — не спрацьовували.

## Зачіпає

`app/package.json`, `bun.lock`, `knip.json`, `.cspell.json`, `.v8rignore`, `.github/workflows/git-ai.yml`.

---

**Опрацьовано** 2026-05-19. Проекції:
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
