---
session: 76bfcf1f-39b2-4d5e-88b6-c387f498efb9
captured: 2026-05-16T16:03:10+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/76bfcf1f-39b2-4d5e-88b6-c387f498efb9.jsonl
---

## ADR Вибір UI-фреймворку MLMaiL: Quasar 2 з macOS material-look

**Контекст:** MLMaiL — Tauri-застосунок для macOS та Android. Правило `.cursor/rules/vue.mdc` вимагало Quasar, але фреймворк ще не був встановлений; Login.vue використовував plain HTML з ручним CSS.

**Рішення/Процедура/Факт:** Введено Quasar 2 через `@quasar/vite-plugin` із sass-vars overrides для macOS material-look (`$primary: #0a84ff`, `system-ui`/SF Pro font stack, `$button-border-radius: 6px`, `$generic-border-radius: 8px`, `dark: 'auto'`). `Login.vue` переписано на `q-page` + `q-btn` + `q-card` + `q-chip` + `q-skeleton` + `q-banner`. Іконки: `@quasar/extras/material-symbols-outlined`. Плагіни `Dialog` і `Notify` — не підключені; помилки показуються inline `q-banner`. `sassVariables` у `vite.config.js` — абсолютний шлях через `fileURLToPath` (без цього sass шукає шлях відносно `node_modules/quasar/src/css/`).

**Обґрунтування:** Quasar дає Material Design «з коробки», що відповідає очікуванням Android-юзерів (ripple, safe-area, touch UX); bundle size (120 КБ gzip) некритичний для Tauri-бінарника (~10 МБ); `Platform` API дозволяє одному компоненту мати дрібні дельти macOS vs Android; майбутні фічі (`q-pull-to-refresh`, `q-virtual-scroll`, `$q.dialog`) підключаться без нової залежності.

**Розглянуті альтернативи:** Plain HTML + scoped CSS (нуль deps, повний контроль — обрано раніше; відкинуто бо потрібен готовий UX на Android); Reka UI (headless primitives + Tailwind, ~10 КБ tree-shaken, максимум контролю — підходить якщо є власна design-system, але потребує написати CSS для всього).

**Зачіпає:** `app/package.json`, `app/vite.config.js`, `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`, `app/src/quasar-variables.sass` (новий), `app/src/test-utils/quasar.js` (новий), `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`, `docs/adr/_inbox/`.

---

## Knowledge `q-page` у vitest: потрібен `q-layout` контекст

**Контекст:** Після переходу Login.vue на `<q-page>` тести зламались (`expected '' to contain 'Увійти через Google'`) — компонент рендерився порожнім, бо `q-page` вимагає батьківського `q-layout`.

**Рішення/Процедура/Факт:** Утиліта `mountWithQuasar` (файл `app/src/test-utils/quasar.js`) обгортає тестований компонент у `h(QLayout, { view: 'hHh lpR fFf' }, () => [h(QPageContainer, () => [h(component)])])` і реєструє Quasar plugin із `{ config: { dark: false } }`. Всі Login-тести перейшли з `mount(Login)` на `mountWithQuasar(Login)`.

**Обґрунтування:** `q-page` використовує ін'єкцію контексту від `q-layout` для визначення розмірів — без нього рендерить порожній `div`. Виявлено емпірично під час запуску vitest після рефакторингу.

**Розглянуті альтернативи:** `attachTo: document.body` — не вирішує проблему ін'єкції; рефактор Login.vue без `q-page` — ламає layout у реальному застосунку.

**Зачіпає:** `app/src/test-utils/quasar.js`, `app/src/views/Login.test.js`.
