---
session: 76bfcf1f-39b2-4d5e-88b6-c387f498efb9
captured: 2026-05-16T16:18:24+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/76bfcf1f-39b2-4d5e-88b6-c387f498efb9.jsonl
---

(answering the question, the previous response ended the flow — no continuation expected)

---

## ADR Введення Quasar 2 як UI-фреймворку MLMaiL з macOS material-look
**Контекст:** MLMaiL — Tauri-застосунок для macOS і Android. Правило `.cursor/rules/vue.mdc` вимагає використовувати Quasar для компонентів, але фреймворк у проєкті був відсутній; Login.vue використовував plain HTML без жодних UI-компонентів.
**Рішення/Процедура/Факт:** Додано залежності `quasar`, `@quasar/extras`, `@quasar/vite-plugin`, `sass`. Налаштовано `@quasar/vite-plugin` у `vite.config.js` з absolute-path `sassVariables` через `fileURLToPath`. Створено `app/src/quasar-variables.sass` — sass-override для macOS material-look: `$primary: #0a84ff` (macOS Accent Blue), system-ui + SF Pro font stack, `$button-border-radius: 6px`, `$generic-border-radius: 8px`. `main.js` реєструє Quasar з `iconSet: 'material-symbols-outlined'` і `dark: 'auto'`. `App.vue` обгорнуто у `<q-layout><q-page-container>`. `Login.vue` переписано на `q-page / q-btn / q-card / q-chip / q-banner / q-skeleton`. Тести адаптовано через `mountWithQuasar` helper у `app/src/test-utils/quasar.js`, що обгортає компонент у Layout-контекст (необхідно для `q-page`). Всі 45 тестів і 64 Rust-тести PASS. Документацію оновлено в `docs/ci4/03-components.md`, `04-code.md`, `decisions.md`.
**Обґрунтування:** Для Tauri macOS + Android Material Design є рідним UX на Android (60% цільової аудиторії). Quasar надає готові компоненти (`q-pull-to-refresh`, `q-virtual-scroll`, `q-skeleton`, `$q.notify`) для майбутніх фіч email-клієнта. Bundle size (~120 КБ gzip) не критичний — Tauri-бінарник і так ~10 МБ. Правило `.cursor/rules/vue.mdc` прямо вказує Quasar. macOS material-look нівелюється через sass-vars (system-ui шрифт, мʼякший radius, iOS Accent Blue як `$primary`).
**Розглянуті альтернативи:** Plain HTML + scoped CSS (поточний стан) — нуль deps, але все UX-браузерне треба писати вручну; Reka UI (Radix Vue) — headless, максимальний контроль, ~10–20 КБ tree-shaken, але нема готових card/spinner/banner, потрібен окремий CSS-кит.
**Зачіпає:** `app/package.json`, `bun.lock`, `app/vite.config.js`, `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`, `app/src/quasar-variables.sass` (новий), `app/src/test-utils/quasar.js` (новий), `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`, `docs/adr/_inbox/`.

## Knowledge git-ai checkpoint: stage-only неможливо, hook виконується на кожну Claude Code операцію
**Контекст:** У глобальному `~/.claude/settings.json` зареєстровано `PreToolUse` і `PostToolUse` hook `*` → `git-ai checkpoint claude`. Через це кожна Edit/Write/Bash операція Claude Code тригерить автоматичний коміт у поточному git-репо.
**Рішення/Процедура/Факт:** git-ai v1.4.8 не має прапорця `--stage-only` або `--no-commit` у команді `checkpoint` — дизайн tool-у передбачає коміт як одиницю AI-authorship трекінгу. Команда оновлення: `curl -fsSL https://usegitai.com/install.sh | bash`. Для відключення в одному репо: `git-ai config --add exclude_repositories <path>`. Для відключення глобально для Claude Code: прибрати `"hooks"` блок із `~/.claude/settings.json`. Альтернатива — підмінити hook на кастомний `git add -A` скрипт (ламає AI-трекінг).
**Обґрунтування:** Поведінка здивувала у сесії — коміти зʼявлялись до виконання відповідної задачі плану. Це не баг, а штатна робота git-ai: checkpoint-hook відслідковує AI-авторство через `refs/notes/ai` прикріплені до SHA комітів.
**Розглянуті альтернативи:** не обговорювалися.
**Зачіпає:** `~/.claude/settings.json` (глобальний), `git log` (помітні checkpoint-коміти), проєкт `mlmail` і будь-який інший репо відкритий у Claude Code.
