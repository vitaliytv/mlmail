# Quasar 2 як UI-фреймворк MLMaiL з macOS material-look

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

MLMaiL — Tauri-застосунок для macOS і Android. `Login.vue` існував на голому HTML + scoped CSS без UI-фреймворку, попри правило `.cursor/rules/vue.mdc`, яке вже передбачає Quasar. Порівнювалися три підходи: plain HTML, Quasar 2 і Reka UI (headless Radix-Vue).

## Рішення/Процедура/Факт

Обрано Quasar 2 з кастомними sass-vars для macOS material-look:

- Встановлено `quasar`, `@quasar/extras`, `@quasar/vite-plugin`, `sass`.
- `app/src/quasar-variables.sass`: `$primary: #0a84ff` (macOS Accent Blue), `$typography-font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui`, `$button-border-radius: 6px`, `$generic-border-radius: 8px`.
- `app/vite.config.js` — `@quasar/vite-plugin` з абсолютним шляхом через `fileURLToPath` для `sassVariables` (відносний шлях не працює — Sass шукає відносно `node_modules/quasar/src/css/`).
- `app/src/main.js` — Quasar з `iconSet: 'material-symbols-outlined'`, `config: { dark: 'auto' }`. Плагіни `Notify`/`Dialog` не підключено.
- `app/src/App.vue` — обгорнуто у `<q-layout view="hHh lpR fFf"><q-page-container>`.
- `app/src/views/Login.vue` — переписано: `q-page`, `q-btn` (`:loading`), `q-chip` для inbox count, `q-card`+`q-separator` для листа, `q-skeleton` для loading-станів, `q-banner` для помилок inline. Logout без confirm-діалогу.
- `app/src/test-utils/quasar.js` — helper `mountWithQuasar` обгортає компонент у `<q-layout><q-page-container>` і реєструє Quasar з `dark: false`. Без цього `q-page` рендерить порожній вивід — компонент вимагає Layout-контексту через CSS custom properties.

## Обґрунтування

Material Design є рідним UX-стандартом на Android. Quasar надає готові `q-pull-to-refresh`, `q-virtual-scroll`, `q-skeleton`, `$q.notify`, `$q.dialog` — компоненти, що знадобляться у майбутніх фічах email-клієнта. Bundle size (~120 КБ gzip) некритичний для Tauri-бінарника (~10 МБ). Правило `.cursor/rules/vue.mdc` прямо закріплює вибір Quasar.

## Розглянуті альтернативи

- **Plain HTML + scoped CSS** — нуль залежностей, але весь mobile UX треба писати вручну; не відповідає правилу.
- **Reka UI (Radix Vue, headless)** — ~10–20 КБ tree-shaken, повний контроль CSS, a11y з коробки; підходить для власної design-системи з Tailwind/shadcn-vue; немає готових card/banner/spinner.

## Зачіпає

`app/package.json`, `bun.lock`, `app/vite.config.js`, `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`, `app/src/quasar-variables.sass` (новий), `app/src/test-utils/quasar.js` (новий), `app/src/views/Login.test.js`, `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`.
