# Changelog

Усі помітні зміни цього пакета документуються тут.

Формат — [Keep a Changelog](https://keepachangelog.com/uk/1.1.0/), нумерація — [SemVer](https://semver.org/lang/uk/).

## [0.1.6] - 2026-06-15

### Added

- Tool Surface (`n-tool-surface`) у `app/src/tool/`: спільний каталог інструментів (`catalog.js`) зі схемами, `dispatch.js` з уніфікованим конвертом `{ ok, output } / { ok:false, error:{ code, message, kind } }`, `manifest.js` (OpenAI function-calling для майбутнього LLM-агента), `transports.js` (Tauri-invoke) та `index.js`. Тести `tool.test.js`.

### Changed

- `auth-store.js` переведено з прямих `invoke` на `dispatch` для read/action-команд (`is_authenticated`, `current_email`, `inbox_count`, `random_message`, `random_newsletter`, `unsubscribe`); lifecycle/secret (`login`, `logout`, `getAccessToken`) лишаються прямими invoke. Конверт зберігає backend-`kind`, тож ReauthRequired-логіка не змінилась.
- Тестовий стек переведено з `bun test` + happy-dom preload на **Vitest** + happy-dom (канон `n-test`/`n-vue`). `vitest.config.mjs` через `mergeConfig` повторно використовує `vite.config.js` (Vue/VueMacros/AutoImport/Quasar), тож SFC-компіляція й авто-імпорти працюють нативно — preload-хаки прибрано. Тест-файли переписано з `bun:test` (`mock`/`mock.module`) на `vitest` (`vi`/`vi.hoisted`/`vi.mock`). `stryker.config.mjs` — на `vitest-runner` з `perTest`-аналізом покриття.

### Removed

- Тестовий preload `test/happy-dom.preload.js` і залежності `@happy-dom/global-registrator`, `@types/bun`, `@vue/compiler-sfc` (більше не потрібні з Vitest).

### Fixed

- Логін з ненастроєними Google OAuth credentials більше не падає з плутаним `OAuth`-помилкою: `run_login` (macOS/Android) перевіряє client_id/secret через `require_configured` ще до старту флоу й повертає новий `ConfigMissing(<env-var>)` → UI показує «Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.». `is_real_client_id` тепер відхиляє і порожній/пробільний рядок (раніше порожній id вважався «справжнім»).

## [0.1.5] - 2026-05-26

### Changed

- Оновлено форматування JS/Vue джерел, тестового preload та Vite-конфігу для узгодження з поточними tooling-перевірками.

## [0.1.4] - 2026-05-26

### Changed

- Посилено JS-тести auth store та нормалізацію повідомлень помилок для кращого mutation coverage.

### Fixed

- Оновлено конфіги lint/coverage, щоб ігнорувати локальні артефакти Stryker і worktree-копії під час перевірок.

## [0.1.3] - 2026-05-22

### Added

- Мутаційне тестування: скрипти `test:mutation` (StrykerJS, JS/Vue) та `test:rust:mutation` (`cargo mutants`, Rust).
- `stryker.config.mjs` — конфігурація StrykerJS: command-runner на `bun test`, `inPlace: true`, мутує `src/**/*.{js,vue}` мінус тести та `main.js`, звіт у `reports/stryker/mutation.json`.

### Fixed

- Порожнє тіло листа для частини повідомлень: Gmail повертає `body.data` як base64url із `=`-паддингом, який код раніше відхиляв. Декодування зроблено толерантним до паддингу, а байти тепер конвертуються в текст за `charset` із заголовка MIME-частини (через `encoding_rs`, fallback — UTF-8), тож листи в ISO-8859-1, windows-1251 тощо більше не показують порожнє тіло.

## [0.1.2] - 2026-05-21

### Added

- Скрипт `test:coverage` (`bun test --coverage`) для звіту покриття JS.
- Скрипт `test:rust:coverage` (`cargo llvm-cov`) для звіту покриття Rust.
- Тести для `App.vue`.

### Changed

- `test-utils/quasar.js`: додано хелпер `mountQuasar` для монтування компонентів із власним layout.

## [0.1.1] - 2026-05-21

### Added

- Тестовий preload `test/happy-dom.preload.js`: реєструє happy-dom як DOM-середовище, компілює `.vue` SFC через Bun-плагін на `@vue/compiler-sfc`, віддає авто-імпорти Vue / Vue Router як глобальні змінні і підміняє `quasar` на browser-збірку.
- Залежності `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`.

### Changed

- Компонентні тести переведено з Vitest на Bun Test Runner + happy-dom.
- `Login.vitest.js` перейменовано на `Login.test.js` і переписано під `bun:test` (`mock` / `mock.module`).
- `mountWithQuasar` реєструє всі Quasar-компоненти глобально (немає `@quasar/vite-plugin` під `bun test`).
- Скрипт `test` запускає `bun test` для всього `src`; додано `test:watch`.

### Removed

- Залежності `vitest` і `jsdom`, блок `test` із `vite.config.js`, скрипт `test:ui`.
