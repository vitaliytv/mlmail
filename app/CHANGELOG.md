# Changelog

Усі помітні зміни цього пакета документуються тут.

Формат — [Keep a Changelog](https://keepachangelog.com/uk/1.1.0/), нумерація — [SemVer](https://semver.org/lang/uk/).

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
