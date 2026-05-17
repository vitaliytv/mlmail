---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-17T07:30:30+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

I'll produce a durable knowledge artifact from this session transcript.

---

## ADR Таурі-тестування на macOS: Mock Runtime замість WebDriver

**Контекст:** Розробляється Tauri v2 десктопний додаток на macOS. Потрібно вирішити стратегію e2e/integration тестування, враховуючи що macOS офіційно не підтримується `tauri-driver` (Apple не надає WebKit WebDriver для WKWebView у вбудованих застосунках).

**Рішення/Процедура/Факт:**
- Вибрано офіційний шлях Tauri v2 — **Mock Runtime** (`tauri::test::mock_builder` + `mock_context` + `noop_assets`) замість WebDriver чи Appium.
- Додано `tauri = { version = "2", features = ["test"] }` у `[dev-dependencies]` у `src-tauri/Cargo.toml`.
- Рефакторинг для DI тестованості: `Box<dyn RefreshTokenStorage>` → `Arc<dyn ...>` як `SharedStorage` (managed Tauri state); новий файл `src-tauri/src/endpoints.rs` містить `Endpoints { google_token, gmail_label_inbox, gmail_messages_list }` — URL як managed state, щоб тести могли підставляти `mockito`-адреси.
- Виділено `finalize_login(resp, &dyn storage, &Mutex<state>)` як публічний хелпер — логіка, що раніше була захована в `auth_start_login`, тепер окремо тестується.
- Написано 19 integration-тестів у `src-tauri/tests/`: `auth_commands.rs`, `auth_logout.rs`, `auth_finalize_login.rs`, `acquire_access_token.rs`, `gmail_commands.rs`.

**Обґрунтування:** Tauri team явно заявляє «On desktop, only Windows and Linux are supported» для WebDriver; macOS без обхідних шляхів (Docker/Linux VM) не має жодного офіційного UI e2e механізму. Mock Runtime покриває рівень `State<>` + `AppHandle`, дозволяє використовувати `mockito` для HTTP-ендпоінтів Google/Gmail і не потребує реального keyring/браузера. Appium + XCUITest розглядався, але є community-solution без офіційної підтримки Tauri.

**Розглянуті альтернативи:**
- `tauri-driver` + WebDriverIO на Linux через Docker/OrbStack (офіційне, але не продова платформа macOS, складне налаштування OAuth-моків)
- Appium + XCUITest driver (нативний macOS UI, але не WebDriver DOM-протокол, нема офіційної підтримки Tauri)
- Playwright проти `bun run dev` без Tauri runtime (лише frontend DX, не e2e)

**Зачіпає:** `src-tauri/Cargo.toml`, `src-tauri/src/auth/mod.rs`, `src-tauri/src/auth/storage/mod.rs`, `src-tauri/src/endpoints.rs` (новий), `src-tauri/src/gmail/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/tests/` (5 нових файлів).

---

## ADR Конвенція тест-раннерів: bun:test для pure-JS, Vitest для Vue SFC

**Контекст:** Проект має фронтенд на Vue 3 + Quasar з `unplugin-auto-import`. `bun test` запускає Bun's власний runner без Vite pipeline, що ламало тести через відсутність auto-import transform і несумісний API (`vi.fn()` vs `mock()`).

**Рішення/Процедура/Факт:**
- `*.test.js` → `bun:test` API (`mock`, `mock.module`, імпорт з `bun:test`). Файли: `auth-store.test.js`, `auth-errors.test.js`.
- `*.vitest.js` → Vitest (Vue SFC + Quasar plugin pipeline). Файл `Login.test.js` перейменовано в `Login.vitest.js`.
- `vite.config.js` `include` звужено до `'src/**/*.vitest.{js,vue}'` — Vitest не підхоплює `*.test.js`.
- `auth-store.js` отримав explicit `import { readonly, ref } from 'vue'` замість auto-import.
- `package.json` скрипти: `test` (bun:test), `test:ui` (vitest), `test:rust` (cargo test), `test:all` (усі три послідовно).

**Обґрунтування:** Vue SFC потребує компіляції через `@vitejs/plugin-vue` + Quasar plugin — за межами bun's вбудованого loader-а. Pure-JS логіка (store, i18n) не залежить від Vite і виграє від швидкості bun:test. Розподіл за розширенням (`.test.js` / `.vitest.js`) дає чітку конвенцію без конфігурації per-file.

**Розглянуті альтернативи:**
- `bunx vitest` для всього (нуль змін коду, але `bun test` все одно не працює)
- Повна міграція на `bun:test` включно з Vue SFC через community `bun-plugin-vue` (нестабільний, ~3-4 год без гарантій)

**Зачіпає:** `app/package.json`, `app/vite.config.js`, `app/src/services/auth-store.js`, `app/src/services/auth-store.test.js`, `app/src/i18n/auth-errors.test.js`, `app/src/views/Login.test.js` → `Login.vitest.js`.

---

## Knowledge Material Symbols Outlined: префікс `sym_o_` обов'язковий з Quasar 2

**Контекст:** Quasar 2 з iconSet `material-symbols-outlined` (підключений через `@quasar/extras/material-symbols-outlined`) генерує CSS-клас `.material-symbols-outlined` лише якщо назва іконки має префікс `sym_o_`. Без префікса Quasar генерує клас `.material-icons` — шрифт не знайдено, браузер відображає сирий текст рядка (наприклад, `mail`, `refresh`, `logout`).

**Рішення/Процедура/Факт:** Іконки в `Login.vue` виправлено: `icon="mail"` → `icon="sym_o_mail"`, аналогічно `refresh`, `logout`, `login`. Префіксація є обов'язковою для всіх Material Symbols Outlined іконок у Quasar 2.

**Обґрунтування:** Quasar 2 підтримує кілька наборів іконок з різними префіксами. `sym_o_` відповідає `material-symbols-outlined`, `sym_r_` — `rounded`, `sym_s_` — `sharp`. Без префікса Quasar вважає іконку як `material-icons` (старий стиль).

**Розглянуті альтернативи:** Підключити обидва шрифти (`material-icons` + `material-symbols-outlined`) — надлишкова вага. Перейти на `material-icons` iconSet — несумісне з `main.js`.

**Зачіпає:** `app/src/views/Login.vue`, `app/src/main.js` (iconSet конфіг), будь-який Vue-компонент що використовує Quasar icon props.
