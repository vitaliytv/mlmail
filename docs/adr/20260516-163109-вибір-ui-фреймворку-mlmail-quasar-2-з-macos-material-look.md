---
session: 76bfcf1f-39b2-4d5e-88b6-c387f498efb9
captured: 2026-05-16T16:31:09+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/76bfcf1f-39b2-4d5e-88b6-c387f498efb9.jsonl
---

Based on the session transcript, here is the captured knowledge artifact:

---

## ADR Вибір UI-фреймворку MLMaiL: Quasar 2 з macOS material-look

**Контекст:** MLMaiL — Tauri-застосунок, що запускається лише на macOS і Android (через Tauri WebView). Перед рефакторингом Login.vue використовував plain HTML/CSS без UI-фреймворку. Потрібно було обрати між трьома варіантами: plain HTML, Quasar 2, Reka UI (headless Radix-Vue).

**Рішення/Процедура/Факт:**
- Обрано **Quasar 2** з кастомними sass-overrides для macOS material-look.
- Встановлено: `quasar`, `@quasar/extras`, `@quasar/vite-plugin`, `sass`.
- `app/src/quasar-variables.sass` — macOS Accent Blue `$primary: #0a84ff`, system-ui/SF Pro font stack, `$button-border-radius: 6px`, `$generic-border-radius: 8px`.
- `app/vite.config.js` — `@quasar/vite-plugin` з absolute-path `sassVariables` (через `fileURLToPath` — інакше Sass шукає шлях відносно `node_modules/quasar/src/css/`).
- `app/src/main.js` — Quasar plugin із `iconSet: 'material-symbols-outlined'`, `config: { dark: 'auto' }` (наслідує OS). Без `Notify`/`Dialog` plugins (не потрібні в поточному обсязі).
- `app/src/App.vue` — обгорнутий у `<q-layout view="hHh lpR fFf"><q-page-container>`.
- `app/src/views/Login.vue` — переписаний: `q-page`, `q-btn` (з built-in `:loading`), `q-chip` для inbox count, `q-card`+`q-separator` для листа, `q-skeleton` для loading-станів, `q-banner` для помилок. Logout без confirm-діалогу (direct call).
- Помилки — inline `<q-banner class="bg-red-1 text-red-9">`, **не** toast/notify.
- `app/src/test-utils/quasar.js` — `mountWithQuasar` helper, що обгортає компонент у `<q-layout><q-page-container>` і реєструє Quasar plugin із `dark: false`. Потрібно тому, що `q-page` вимагає Layout-контексту; без обгортки тести падають зі SyntaxError.

**Обґрунтування:** Material Design є рідним UX на Android, де більшість мобільних юзерів. Quasar дає готові `q-btn` (loading-стан, ripple, hover), `q-card`, `q-chip`, `q-skeleton`, `$q.notify`/`$q.dialog` (знадобляться у майбутніх фічах). Bundle size (~120 КБ gzip) не критичний для Tauri-бінарника (~10 МБ). Quasar вже вимагається правилом `.cursor/rules/vue.mdc`. macOS material-look «пом'якшено» через sass-vars (system-ui font, менший radius, macOS Accent Blue) — не нативний SF design, але прийнятний компроміс за ціною 20 рядків sass.

**Розглянуті альтернативи:**
- **Plain HTML/CSS** — нульовий bundle, але всі loading-стани, dialogs, toasts, ripple — вручну. Прийнятно якщо UI мінімальний і не росте.
- **Reka UI (Radix Vue, headless)** — headless primitives для a11y (AlertDialog, Tooltip, Toast). Повний контроль над CSS, ~10–20 КБ tree-shaken. Але немає готових button/card/banner/spinner — треба писати власний design-system. Добре під Tailwind/shadcn-vue.
- **Quasar** — обрано. Найкраще під Tauri macOS+Android за співвідношенням готового UX до витраченого часу.

**Зачіпає:**
- `app/package.json`, `bun.lock`
- `app/vite.config.js`
- `app/src/main.js`
- `app/src/App.vue`
- `app/src/views/Login.vue`
- `app/src/quasar-variables.sass` (новий файл)
- `app/src/test-utils/quasar.js` (новий файл)
- `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`
