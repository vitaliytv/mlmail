---
session: 76bfcf1f-39b2-4d5e-88b6-c387f498efb9
captured: 2026-05-16T16:05:39+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/76bfcf1f-39b2-4d5e-88b6-c387f498efb9.jsonl
---

Ось артефакти цієї сесії:

---

## ADR Quasar 2 як UI-фреймворк MLMaiL з macOS material-look

**Контекст:** MLMaiL — Tauri-застосунок для macOS + Android. Login.vue використовував plain HTML без жодної design-системи. Правило `.cursor/rules/vue.mdc` вимагає Quasar, але він ще не був підключений.

**Рішення/Процедура/Факт:** Встановлено `quasar`, `@quasar/extras`, `@quasar/vite-plugin`, `sass`. `vite.config.js` підключає `@quasar/vite-plugin` з `sassVariables: fileURLToPath(new URL('src/quasar-variables.sass', import.meta.url))` (абсолютний шлях — без нього Sass шукав файл відносно `node_modules/quasar/src/css/`). `main.js` реєструє `Quasar` з `dark: 'auto'` та `iconSet: 'material-symbols-outlined'`. `App.vue` обгорнутий у `<q-layout> + <q-page-container>`. `Login.vue` переписаний: `q-btn`, `q-chip`, `q-card`, `q-skeleton`, `q-banner`. Sass-vars macOS-дельти: `$primary: #0a84ff`, system-ui + SF Pro font stack, `$button-border-radius: 6px`. Помилки — inline `<q-banner>` (не toast). Logout без confirm-діалогу. Тести оновлені через `mountWithQuasar` helper, що обгортає компонент у `<q-layout><q-page-container>` (q-page вимагає Layout-контекст, без обгортки тести падають з порожнім render).

**Обґрунтування:** Android — основний mobile-target, де Material Design є системним UX-стандартом (ripple, safe-area, bottom-sheet). Quasar дає `q-pull-to-refresh`, `q-virtual-scroll`, `q-skeleton` — компоненти, що знадобляться при розвитку списку листів. Розмір bundle (~120 КБ gz) некритичний у Tauri-бінарнику (~10 МБ). macOS look досягається мінімально через sass-vars, без переписування компонентів. Reka UI і plain HTML відхилені: перший не дає готових card/banner/chip (треба писати CSS самому), другий не відповідає правилу `.cursor/rules/vue.mdc`.

**Розглянуті альтернативи:** Plain HTML (поточний стан — відхилено: нема готових UX-примітивів, не відповідає правилу); Reka UI (headless, підходить для кастомного design-system з Tailwind — відхилено: занадто багато CSS ручної роботи для поточного розміру команди).

**Зачіпає:** `app/package.json`, `app/vite.config.js`, `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`, `app/src/quasar-variables.sass`, `app/src/test-utils/quasar.js`, `app/src/views/Login.test.js`, `docs/ci4/03-components.md`, `docs/ci4/04-code.md`, `docs/ci4/decisions.md`

---

## Knowledge git-ai глобальний hook автоматично комітить зміни у Claude Code

**Контекст:** Файли в репо комітились автоматично після кожного Edit/Write Claude-агента, ще до того як агент доходив до explicit commit-кроку в плані. Користувач очікував побачити diff і самостійно вирішити що комітити.

**Рішення/Процедура/Факт:** У `~/.claude/settings.json` глобально налаштований hook `git-ai checkpoint claude --hook-input stdin` на події `PreToolUse: *` і `PostToolUse: *`. Він запускається перед і після **кожного** виклику інструменту (Edit, Write, Bash тощо) і робить автоматичний checkpoint-коміт. Це зовнішній процес, не частина Claude-агента. Щоб вимкнути: (A) локально — додати `{"hooks": {"PreToolUse": [], "PostToolUse": []}}` у `<repo>/.claude/settings.json`; (B) глобально — прибрати блок `hooks` із `~/.claude/settings.json`.

**Обґрунтування:** Виявлено через `git log --oneline` — коміти зʼявлялися до того, як агент їх явно робив. Підтверджено читанням `~/.claude/settings.json` і `.claude/hooks/capture-decisions.log`.

**Розглянуті альтернативи:** Не обговорювались — це discovery, а не вибір.

**Зачіпає:** `~/.claude/settings.json` (глобально), усі проєкти де активний Claude Code, `.claude/hooks/` у поточному репо (лог)
