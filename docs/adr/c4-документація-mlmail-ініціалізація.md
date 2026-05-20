# Ініціалізація C4-документації MLMaiL у docs/ci4/

**Status:** Accepted
**Date:** 2026-05-11

## Контекст

Проєкт MLMaiL (Tauri 2 + Vue 3, macOS і Android) не мав жодної архітектурної документації, хоча правило `.cursor/rules/n-ci4.mdc` вимагає C4-діаграм поряд із кодом і встановлює «Rebuild Test»: якщо видалити `src/` і дати агенту лише `.md`-файли — він має відновити кодову базу.

## Рішення/Процедура/Факт

Створено `docs/ci4/` із шістьма файлами на основі реального коду (`lib.rs`, `tauri.conf.json`, `vite.config.js`, `package.json`):

- `README.md` — навігація і статус;
- `01-context.md` — System Context: користувач, Google Identity Services, Gmail API, LLM/TTS-провайдери;
- `02-containers.md` — MLMaiL Frontend (Vue 3/Vite), MLMaiL Backend (Rust/Tauri 2), WebView, локальне сховище; два варіанти розгортання;
- `03-components.md` — компоненти Vue та Rust із поточним і `planned` статусом;
- `04-code.md` — ключові файли і сигнатури;
- `decisions.md` — 5 прийнятих рішень і 6 відкритих питань для майбутніх ADR.

Виправлено транслітерації у `.cspell.json` (додано `саммері`, `PKCE`, `фронтенд`, `рантайм` та ін.) і два друкарські помилки у `.md`-файлах. Результат: `markdownlint-cli2` — 0 errors, `cspell` — 0 issues. Команда `npx @nitra/cursor check ci4` повертає `❌ Невідомі правила: ci4` — правило відсутнє у CLI `@nitra/cursor`; валідація проводиться через markdownlint та cspell.

## Обґрунтування

Документація розрізняє поточний стан (каркас з `greet`) і `planned`-компоненти (Gmail-інтеграція, LLM, TTS), щоб уникнути дезінформації агентів. Структура `01–04 + README + decisions` є канонічною за правилом `n-ci4.mdc`.

## Розглянуті альтернативи

Не обговорювалися — розташування і структура визначені правилом `n-ci4.mdc`.

## Зачіпає

`docs/ci4/README.md`, `docs/ci4/01-context.md`, `docs/ci4/02-containers.md`, `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`, `.cspell.json`.

---

**Опрацьовано** 2026-05-20. Проекції:
- [01-context](../ci4/01-context.md)
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
