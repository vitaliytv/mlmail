---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-16T16:09:14+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

## Knowledge Tauri v2 e2e на macOS — WebDriver офіційно не підтримується

**Контекст:** Розглядалась можливість запускати e2e-тести для Tauri v2 застосунку (MLMaiL) безпосередньо на macOS без CI/Docker.

**Рішення/Процедура/Факт:** Tauri v2 офіційно не підтримує WebDriver на macOS — Apple не надає WKWebView WebDriver tool. Офіційна документація (https://v2.tauri.app/develop/tests/webdriver/) прямо вказує: `"On desktop, only Windows and Linux are supported due to macOS not having a WKWebView driver tool available."` Тобто на macOS офіційно доступні лише: (1) unit-тести через `tauri::test::mock_builder()`, (2) integration-тести через `tauri::test::mock_app()`, (3) Vitest/компонентні тести для Vue-фронту з `vi.mock('@tauri-apps/api/core')`.

**Обґрунтування:** Системне обмеження macOS, не проблема Tauri — Apple не експонує WebKit WebDriver для WKWebView у вбудованих застосунках (лише для Safari через `safaridriver`).

**Розглянуті альтернативи:** (1) **Appium mac2-driver** — драйверить UI через macOS Accessibility API (AX-дерево), не DOM-селектори; неофіційно але технічно працює; тестує реальний keyring apple-native; (2) **Docker + Xvfb на Linux** — справжній WebDriver, але збирає Linux-бінарник, keychain/apple-native не тестується; (3) **Docker + noVNC** — те саме що (2), але з видимим вікном у браузері.

**Зачіпає:** `app/src-tauri/`, `app/package.json`, будь-яка e2e-інфраструктура проєкту MLMaiL.

---

## ADR e2e-тестування MLMaiL: вибір між Appium mac2 і Linux WebDriver

**Контекст:** MLMaiL — Tauri v2 застосунок з Google OAuth + Gmail API, основна платформа — macOS. Необхідно вирішити, де і як запускати реальний e2e (повний бінарник + UI + WebDriver-протокол або аналог).

**Рішення/Процедура/Факт:** Сесія завершилась без фінального вибору, але сформульовано два реальні варіанти: (A) **Appium mac2** — `appium driver install mac2` + WDIO з `'appium:automationName': 'mac2'`; видиме вікно, нативний keyring, ~4 год setup, AX-селектори замість DOM; (B) **Docker + Linux WebDriver** — ubuntu-образ з `webkit2gtk-driver` + `xvfb` (+опційно noVNC), `tauri-driver`, WDIO з DOM-селекторами; ~2–3 год на Docker image. В обох варіантах **обов'язково** виносити Google token URL і Gmail API base у env-конфіг Rust + підіймати локальний HTTP OAuth-стаб, бо реальний Google OAuth у автотесті непрохідний (2FA, captcha).

**Обґрунтування:** Варіант A тестує продову платформу (macOS + apple keychain), не потребує Docker; варіант B дає справжній WebDriver-протокол (DOM-селектори, CSS/XPath) та відповідає офіційному шляху Tauri. Вибір залежить від того, чи критично тестувати keyring і macOS-специфічну поведінку.

**Розглянуті альтернативи:** Playwright проти `vite dev` (без Tauri runtime) — швидко і кросплатформенно, але це інтеграція фронту, не e2e Tauri-бінарника. Посилення Vitest (юніти + `mockito` для Rust) — найдешевше і рекомендоване для 90% покриття, але не задовольняє вимогу "реальний UI + WebDriver".

**Зачіпає:** `app/src-tauri/src/auth/`, `app/src-tauri/src/gmail/`, `app/src-tauri/Cargo.toml`, `app/package.json`, нова директорія `app/e2e/` (WDIO-конфіг і спеки).
