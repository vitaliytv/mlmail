# Changelog

## [0.7.0] - 2026-07-12

### Fixed

- Fix pre-existing eslint/oxlint errors across the codebase (JSDoc, static regex, no-alert, etc.)

## [0.6.0] - 2026-07-12

### Fixed

- Filters dialog: show all Gmail filter conditions, hide raw id, add search and per-filter action icons

## [0.5.2] - 2026-07-11

### Changed

- У діалозі «Правило для схожих листів»: поки локальна LLM підбирає тему, замість кружечка одразу показується хрестик, що скасовує підбір і очищає поле теми. Кнопка «Видалити всі такі» тепер формує Gmail-запит лише за відправником, без теми — щоб масове видалення не залежало від підказаної LLM фрази.

## [0.5.1] - 2026-07-11

### Fixed

- Автооновлення насправді ніколи не показувало діалог: перевірка знаходила нову версію й навіть завантажувала її, але виклик `$q.dialog(...)` падав з `TypeError: e.dialog is not a function` — Quasar-плагін `Dialog` не був підключений у `main.js` (лише `Notify`). Помилка тихо ловилась у catch механізму оновлення й ніде не показувалась користувачу.

## [0.5.0] - 2026-07-06

### Changed

- Додано діалог для перегляду та видалення фільтрів Gmail

## [0.4.5] - 2026-07-06

### Fixed

- Виправлено визначення кодування HTML/plain-text листів: помилково задекларований легасі charset (напр. windows-1251) більше не спричиняє мojibake, якщо сирі байти насправді валідний UTF-8

## [0.4.4] - 2026-07-05

### Fixed

- Android-білд падав ("Permission updater:default not found") — updater-плагін не реєструється на Android/iOS, тож дозвіл винесено в окрему capability з `platforms: [macOS, windows, linux]`.

## [0.4.3] - 2026-07-05

### Changed

- Локальний use-updater.js замінено на спільний useUpdater() з @7n/tauri-components/vue (0.8.0) — та сама логіка, тепер в одній копії для mlmail/myshare/myllm/task.

## [0.4.2] - 2026-07-05

### Added

- Автоматичний повторний запит до Gmail кожні 15 секунд при мережевій помилці (kind: Network), з нотифікацією користувача про кожну спробу — замість негайного відображення помилки "Не вдалося з'єднатися з Google".

### Fixed

- Автооновлення не працювало: у capabilities/default.json бракувало дозволу `updater:default`, тож `check()` завжди падав з permission-denied ще до мережевого запиту (тихо ловилось у catch, консоль недоступна в релізній збірці). Додано дозвіл.

## [0.4.1] - 2026-07-05

### Changed

- release: app@0.4.0

## [0.4.0] - 2026-07-05

### Added

- Аналіз дзвінків: новий компонент AuditAnalysisDialog.vue з use-call-analysis.js для аналізу записів розмов, backend-сервіс call_analysis.rs у Tauri
- Розширена каталогізація інструментів у tool/catalog.js

### Changed

- Оновлено NewsletterView.vue й TasksPanel.vue для інтеграції з новим аналізом

## [0.3.0] - 2026-07-05

### Added

- Перезапуск у нову версію одразу після встановлення оновлення (relaunch), періодична перевірка оновлень щогодини, логування помилок апдейтера

### Fixed

- Апдейтер не запускається в dev-режимі: версія dev-збірки завжди 0.1.0, тож перевірка помилково пропонувала «оновитись» до опублікованого релізу

## [0.2.0] - 2026-07-03

### Added

- Правило для схожих листів: на картці листа кнопка «Правило» відкриває панель із полями Від (email)/Тема (стабільний префікс підказує локальний LLM, редаговано) і двома діями — «Видалити всі такі» (gmail_trash_query: пагінований збір усіх збігів + batchModify TRASH батчами ≤1000) та «Створити фільтр» (gmail_create_filter: Gmail-фільтр з action addLabelIds TRASH/removeLabelIds INBOX). Новий OAuth-скоуп gmail.settings.basic (потрібен повторний логін). Tool-каталог: trash_query (destructive), create_filter (write).
- Двоколонковий рідер листа: ліворуч оригінал (Від/Тема/Дата + тіло), праворуч резюме українською від локального LLM (omlx) — composables/use-summary.js + pure services/summary.js (buildSummaryPrompt). Резюме автоматично оновлюється при зміні листа (watch на currentMessage) із захистом від застарілих відповідей; на недоступній моделі показує банер-підказку. Колонки адаптивні (col-12 col-md-6).
- Панель задач (tasks panel): use-task-scan.js сканує вхідні на предмет задач, use-ask.js для швидких запитів до LLM, нові компоненти в src/components/
- Версія застосунку відображається в заголовку вікна (mlmail vX.Y.Z)
- Шаблони для новинних розсилок (newsletter): JSON-шаблони пакуються в бінарник як bundle-ресурс, use-newsletter-render.js рендерить лист за шаблоном, TemplatesManager.vue керує списком

### Fixed

- Авто-оновлення: увімкнено bundle.createUpdaterArtifacts — релізи тепер публікуватимуть latest.json і .sig
- Резюме/переклад листа: fetch з abort-таймаутом 60с (omlx міг зависати без відповіді) і зменшений розмір батчу перекладу (80→15) для стабільної якості й швидкості

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
