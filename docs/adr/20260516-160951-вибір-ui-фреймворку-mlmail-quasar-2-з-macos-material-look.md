---
session: 76bfcf1f-39b2-4d5e-88b6-c387f498efb9
captured: 2026-05-16T16:09:52+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/76bfcf1f-39b2-4d5e-88b6-c387f498efb9.jsonl
---

---

## ADR Вибір UI-фреймворку MLMaiL: Quasar 2 з macOS material-look

**Контекст:** MLMaiL — Tauri-застосунок, що підтримується лише на macOS та Android. `Login.vue` існував на голому HTML + scoped CSS без жодного UI-фреймворку, попри вже наявне правило в `.cursor/rules/vue.mdc` «Використовуй Quasar для компонентів». Потрібно було вибрати та впровадити UI-фреймворк, який органічно вписується в macOS + Android і дає достатній UI-kit для email-застосунку.

**Рішення/Процедура/Факт:** Обрано Quasar 2 з кастомними sass-vars для macOS material-look. Виконано: (1) додано `quasar`, `@quasar/extras`, `@quasar/vite-plugin`, `sass` як залежності у `app/package.json`; (2) створено `app/src/quasar-variables.sass` — `$primary: #0a84ff` (macOS Accent Blue), `system-ui`/SF Pro font stack, `$button-border-radius: 6px`, `$generic-border-radius: 8px`; (3) `app/vite.config.js` — `@quasar/vite-plugin` з absolute path через `fileURLToPath` для `sassVariables` (інакше Sass шукає шлях відносно `node_modules/quasar/src/css/`) + `transformAssetUrls` у Vue plugin; (4) `app/src/main.js` — реєстрація `Quasar` з `iconSet: 'material-symbols-outlined'`, `config.dark: 'auto'`; (5) `App.vue` обгорнутий у `<q-layout> + <q-page-container>`; (6) `Login.vue` переписаний на `q-page` + `q-btn` + `q-card` + `q-chip` + `q-banner` + `q-skeleton`; (7) helper `app/src/test-utils/quasar.js` (`mountWithQuasar`) обгортає компонент у `q-layout + q-page-container` для vitest — бо `q-page` вимагає Layout-контексту.

**Обґрунтування:** Material Design = рідний UX на Android; `$primary: #0a84ff` + system-ui font stack + менший radius наближають вигляд до macOS без повного відходу від Material; bundle ~120 КБ gzip не критичний для Tauri-застосунку (де бінарник ~10 МБ); готові `q-dialog`, `$q.notify`, `q-pull-to-refresh`, `q-virtual-scroll` будуть потрібні в майбутніх фічах (список листів, AI-саммері, нотатки); правило `.cursor/rules/vue.mdc` вже закріплює вибір Quasar. Logout-confirm та notify-toast відкинуто як передчасна ускладненість — лишили inline `q-banner` і прямий logout.

**Розглянуті альтернативи:** Plain HTML + scoped CSS (нуль deps, але весь material/mobile UX треба писати вручну, нема готових діалогів і спінерів); Reka UI (headless, headless + Tailwind/shadcn-vue, ~10–20 КБ tree-shaken, повний контроль над CSS, ідеально для власної design-системи — але нема готових card/banner/spinner, треба писати CSS самостійно).

**Зачіпає:** `app/package.json`, `bun.lock`, `app/vite.config.js`, `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`, `app/src/quasar-variables.sass` (new), `app/src/test-utils/quasar.js` (new), `app/src/views/Login.test.js`, `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`, `docs/adr/_inbox/20260516-160045-quasar-ui.md`
