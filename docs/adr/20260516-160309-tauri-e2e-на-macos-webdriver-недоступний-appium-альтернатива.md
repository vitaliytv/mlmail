---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-16T16:03:09+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

## Knowledge Tauri e2e на macOS: WebDriver недоступний, Appium — альтернатива

**Контекст:** Проєкт MLMaiL — Tauri v2 + Vue 3 + Quasar, основна платформа macOS. Виникло питання як налаштувати e2e-тести з реальним UI + WebDriver протоколом.

**Рішення/Процедура/Факт:** `tauri-driver` (офіційний WebDriver для Tauri) на macOS **не підтримується** — Apple не надає WebKit WebDriver для WKWebView у вбудованих застосунках (лише `safaridriver` для Safari). Єдиний варіант e2e з DOM-селекторами — Linux-збірка у Docker/VM. Найближча до WebDriver альтернатива на macOS — **Appium + mac2 driver**, який драйверить UI через macOS Accessibility API (`AXLabel`, `AXRole`), без CSS/XPath. Незалежно від підходу — реальний Google OAuth у тестах непридатний (2FA, captcha); потрібно виносити `GOOGLE_TOKEN_URL` та `GMAIL_API_BASE` у env-конфіг Rust і піднімати локальний HTTP-стаб.

**Обґрунтування:** Appium на macOS тестує реальний Mac-бінарник включно з `keyring apple-native` (Keychain), що неможливо відтворити в Linux-збірці. Налаштування швидше (~4 год vs ~6 год для Docker-підходу), не потрібен VM, вікно видиме нативно. DOM-селекторів нема, але Quasar/Vue автоматично експонують текст і role в AX-дерево; де текст динамічний — рекомендовано додавати `aria-label`.

**Розглянуті альтернативи:**
- Docker + Xvfb (headless, Linux-бінарник) — WebDriver-протокол є, але не тестує macOS keyring
- Docker + noVNC (видиме вікно через браузер) — зручніше для ручного запуску, але той самий Linux-бінарник
- Vitest + `@vue/test-utils` + mock `invoke()` — вже є, покриває ~95% багів фронту без жодного WebDriver
- Playwright проти Vite dev-server — не e2e, фронт без Tauri runtime

**Зачіпає:** `app/src-tauri/src/auth/`, `app/src-tauri/src/gmail/`, `app/src-tauri/Cargo.toml` (mockito), `app/src/views/Login.vue` (потребує `aria-label` на динамічних кнопках), `app/package.json` (скрипт `e2e:mac`)
