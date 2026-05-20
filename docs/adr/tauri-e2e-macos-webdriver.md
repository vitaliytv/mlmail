# Tauri e2e на macOS: WebDriver офіційно не підтримується

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

Розглядалася можливість запуску e2e-тестів для Tauri v2-застосунку MLMaiL на macOS через WebDriver-протокол.

## Рішення/Процедура/Факт

Офіційний e2e-стек Tauri — `tauri-driver` + WebDriverIO/Selenium. На macOS підтримки немає: Apple не надає публічний WebKit WebDriver для WKWebView у вбудованих застосунках. Документація v2.tauri.app прямо вказує: «On desktop, only Windows and Linux are supported due to macOS not having a WKWebView driver tool available.» На macOS e2e покриваються через: юніт-тести Rust через `tauri::test::mock_builder()` та Vitest-тести для Vue з `vi.mock('@tauri-apps/api/core')` (без Tauri runtime).

## Обґрунтування

Обмеження платформне: Apple не публікує WebKit WebDriver API для WKWebView (лише `safaridriver` для Safari). Реальні e2e з WebDriver гоняться лише в Linux CI (`ubuntu-latest` + `xvfb-run` + `webkit2gtk-driver`).

## Розглянуті альтернативи

- Appium + mac2 driver — AX-дерево замість DOM-селекторів; неофіційно, без підтримки Tauri; тестує рідний keychain.
- Docker + Xvfb на Linux — справжній WebDriver, але Linux-бінарник, macOS keychain не тестується.

## Зачіпає

CI-конфігурація (`.github/workflows/`), тестова інфраструктура (`wdio.conf.js`, `tauri-driver`), майбутні e2e-сьюти для Tauri-частини MLMaiL.

---

**Опрацьовано** 2026-05-20. Проекції:
- [01-context](../ci4/01-context.md)
- [02-containers](../ci4/02-containers.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
