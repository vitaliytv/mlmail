# Quasar UI-фреймворк з macOS material-look — design spec

**Дата:** 2026-05-16
**Статус:** Approved, готовий до плану імплементації
**Scope:** Ввести Quasar 2 як UI-фреймворк MLMaiL і переписати `Login.vue` на Quasar-компоненти зі стилем «macOS material-look» (system-ui font, мʼякший radius, macOS Accent Blue). Без зміни UX-поведінки.

## Мета

1. Виконати правило `.cursor/rules/vue.mdc` («Використовуй Quasar для компонентів»).
2. Налаштувати Quasar у складі контейнера MLMaiL Frontend через офіційний `@quasar/vite-plugin` і кастомну sass-таблицю змінних `quasar-variables.sass` із macOS-tuned значеннями.
3. Переписати `Login.vue` на Quasar-компоненти (`q-page`, `q-btn`, `q-card`, `q-chip`, `q-banner`, `q-skeleton`) **без зміни UX**: ті ж самі тексти, ті ж самі стани, ті ж самі трігери. Тільки візуальна заміна + skeleton для loading.
4. Зберегти всі поточні Vue-тести робочими через спільний `mountWithQuasar` helper.
5. Зафіксувати рішення «UI = Quasar» у C4-моделі та новому ADR-inbox.

Що **поза scope** цієї ітерації:

- `$q.dialog` для confirm-logout (юзер сказав logout одразу).
- `$q.notify` toast-и (юзер сказав inline q-banner).
- `q-layout` з drawer / header / footer / floating buttons.
- `q-pull-to-refresh` для INBOX (немає списку поки).
- Status bar tint на Android (`Platform.is.android` колоризація).
- UI-перемикач light/dark (dark = `auto` за OS, як зараз).
- Custom splash screen.
- Material 3 dynamic color / Material You.
- Tauri mobile plugin для native-діалогів (`@tauri-apps/plugin-dialog`).
- Інші Vue-файли поза `App.vue` і `Login.vue` (їх ще нема).
- Перейменування `auth-store.js` → `mail-store.js`.

## Ключові архітектурні рішення

1. **Quasar 2 як standalone framework (не Quasar CLI).** Підключаємо через `@quasar/vite-plugin` у наявний Vite-конфіг — не переходимо на `quasar create`. Це зберігає Tauri-pipeline (`vite.config.js`, `vite-plugin-vue-layouts-next`, `unplugin-auto-import`, `vue-macros`) без розриву.
2. **Іконки — Material Symbols Outlined** через `@quasar/extras`. Один CSS + один web-font, ~150 КБ у bundle, але стабільні імена і Material 3 outlined look (тонкі лінії, не Google-3D).
3. **Dark mode — `auto`** через `framework.config.dark: 'auto'`. Наслідує `prefers-color-scheme`, як поточне рішення в `App.vue`.
4. **macOS material-look — через `quasar-variables.sass`**, не runtime-перемикачі. Один sass-файл змінює defaults для всього застосунку: SF-family font stack, мʼякший radius (`6px` для кнопок, `8px` generic), macOS Accent Blue для `$primary` (`#0a84ff`). Це і на Android-у виглядає прийнятно (не material-3 by-the-book, але охайно).
5. **UX не змінюємо.** Той самий перелік станів і той самий потік. Помилки — inline `<q-banner>`, не toast. Logout — одразу, без confirm-діалогу.
6. **Тести через `mountWithQuasar` helper.** Загальна фабрика у `app/src/test-utils/quasar.js`, яка обгортає `mount` з Vue Test Utils, передаючи `global.plugins: [[Quasar, { iconSet, config }]]`. Існуючі тести оновлюємо точково — замінюємо `mount(Login)` на `mountWithQuasar(Login)`. Текстові assertions залишаються.

## Архітектура

### Залежності

`app/package.json`:

- production: `quasar`, `@quasar/extras`.
- dev: `@quasar/vite-plugin`, `sass`.

Не міняємо `vue`, `@vitejs/plugin-vue` тощо — вони залишаються.

### Setup-шар

**`app/vite.config.js`** — додаємо `quasar` plugin у масив:

```js
import { quasar, transformAssetUrls } from '@quasar/vite-plugin'

// у plugins[] перед AutoImport:
quasar({ sassVariables: 'src/quasar-variables.sass' })

// vue plugin отримує transformAssetUrls:
Vue({ template: { transformAssetUrls } })
```

**`app/src/quasar-variables.sass`** — нові sass-defaults:

```sass
// macOS material-look
$primary:   #0a84ff   // macOS Accent Blue
$secondary: #5e5ce6
$accent:    #ff453a
$dark:      #1c1c1e
$dark-page: #000

$typography-font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Inter', system-ui, sans-serif

$button-border-radius:  6px
$generic-border-radius: 8px
```

**`app/src/main.js`** — реєстрація Quasar:

```js
import { Quasar } from 'quasar'
import 'quasar/src/css/index.sass'
import '@quasar/extras/material-symbols-outlined/material-symbols-outlined.css'
import App from './App.vue'

createApp(App)
  .use(Quasar, {
    iconSet: 'material-symbols-outlined',
    config: { dark: 'auto' }
  })
  .mount('#app')
```

**`app/src/App.vue`** — стає тонкою обгорткою без scoped CSS:

```vue
<script setup>
import Login from './views/Login.vue'
</script>

<template>
  <q-layout view="hHh lpR fFf">
    <q-page-container>
      <Login />
    </q-page-container>
  </q-layout>
</template>
```

Старе `:root { ... }` і `@media (prefers-color-scheme: dark)` блоки прибираємо — Quasar бере на себе через `dark: 'auto'`.

### `Login.vue` — переписаний

Корінь — `<q-page>` із flex-центрацією через Quasar utility classes. Кожен існуючий блок мапиться 1-to-1 на Quasar-компонент:

| Сьогодні                                          | Quasar                                                                                                                                               | Нотатка                                          |
| ------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `<h1>MLMaiL</h1>`                                 | `<div class="text-h5 q-mb-md">MLMaiL</div>`                                                                                                          | text-h4/h5 — Quasar typography scale             |
| `<p>Ви увійшли як …</p>`                          | `<div class="text-body1">`                                                                                                                           | без обгортки в компонент                         |
| `<p class="inbox-count">Листів у скриньці: N</p>` | `<q-chip icon="mail" color="primary" text-color="white">`                                                                                            | inline для consistency                           |
| `<p class="inbox-count muted">…</p>` (loading)    | `<q-skeleton type="QChip" width="180px" />`                                                                                                          | новий UX-патерн                                  |
| `<p class="error">` для inbox-помилки             | `<q-banner class="bg-red-1 text-red-9 inline-banner" rounded dense>`                                                                                 | inline помилка                                   |
| `<section class="message">`                       | `<q-card flat bordered class="message-card">` із двома `<q-card-section>` і `<q-separator />` між ними                                               | message-body — `<pre>` всередині другого section |
| `<p class="muted">Завантаження…</p>`              | `<q-skeleton type="text" />` × 5 рядків                                                                                                              | skeleton lines                                   |
| `<button>Показати інший</button>`                 | `<q-btn color="primary" icon="refresh" :loading="auth.isMessageLoading.value">Показати інший</q-btn>`                                                | `<template #loading>` дає custom loading text    |
| `<button>Вийти</button>`                          | `<q-btn flat color="grey-8" icon="logout" @click="auth.logout()">Вийти</q-btn>`                                                                      | без confirm-dialog                               |
| `<button>Увійти через Google</button>`            | `<q-btn color="primary" icon-right="login" :loading="auth.isLoading.value">Увійти через Google` + `<template #loading>Зачекайте…</template></q-btn>` | loading-стан вбудований                          |

Помилки `auth.errorKind` (при login-failure) — також `<q-banner>` нижче кнопки логіну.

Стилі `<style scoped>` для `.message-card` (max-width 60ch) і `.message-body` (white-space pre-wrap, overflow-wrap anywhere) зберігаємо. Решту прибираємо — Quasar typography і spacing utility-класи (`q-gutter-md`, `q-mb-md`, `q-pa-md`) замінюють старий `.login` flex-стан.

### `auth-store.js`

Без змін — UI-фреймворк рефакторинг не торкає бізнес-логіку.

### Тести

`app/src/test-utils/quasar.js` — нова фабрика:

```js
import { mount } from '@vue/test-utils'
import { Quasar } from 'quasar'

export function mountWithQuasar(component, options = {}) {
  return mount(component, {
    ...options,
    global: {
      ...(options.global || {}),
      plugins: [...(options.global?.plugins || []), [Quasar, { config: { dark: false } }]]
    }
  })
}
```

`Login.test.js` — замінити всі `mount(Login)` на `mountWithQuasar(Login)`. Текстові assertions (`toContain('Ви увійшли як …')`, `toContain('Листів у скриньці: 348')`, `toContain('Скринька порожня.')`) — зостаються, бо рендериться той самий текст всередині Quasar-компонентів. Селектори кнопок `findAll('button')` працюють — `q-btn` рендериться як `<button>`.

Один новий тест: skeleton при loading — перевірити, що при `inboxCount === null` присутній `q-skeleton` (через `w.findComponent({ name: 'QSkeleton' })`).

vitest-конфіг (`vite.config.js`): без змін. Sass-vars підвантажуються через quasar-плагін; у тестах CSS не виконується.

### Документація

1. `docs/ci4/03-components.md` — додаємо компонент Frontend UI Kit MLMaiL (Quasar), оновлюємо Auth Component MLMaiL (тепер залежить від Quasar), додаємо примітку про macOS material-look у `quasar-variables.sass`.
2. `docs/ci4/04-code.md` — нові секції: `app/src/quasar-variables.sass`, оновлений `app/src/main.js`, оновлений `app/vite.config.js` (quasar plugin), оновлений `app/src/App.vue` (q-layout), оновлений `app/src/views/Login.vue` (Quasar-компоненти).
3. `docs/ci4/decisions.md` — нове прийняте рішення «UI = Quasar 2 + macOS material-look».
4. ADR-inbox-нотатка `docs/adr/_inbox/<timestamp>-quasar-ui.md`.

## Потік даних

Без змін. Quasar — лише презентаційний шар. `auth-store` API (`login`, `logout`, `refreshInboxCount`, `loadRandomMessage`, ref-и) лишається ідентичним.

## Помилки

| Сценарій                        | Існуюча обробка                     | Зміна після Quasar                                     |
| ------------------------------- | ----------------------------------- | ------------------------------------------------------ |
| `auth.errorKind` (login fail)   | inline `<p class="error">`          | inline `<q-banner class="bg-red-1 text-red-9">`        |
| `auth.inboxErrorKind`           | inline `<p class="error">`          | inline `<q-banner class="bg-red-1 text-red-9">`        |
| `auth.messageErrorKind`         | inline `<p class="error">`          | inline `<q-banner class="bg-red-1 text-red-9">`        |
| `ReauthRequired` (з auth-store) | переходимо на «Увійти через Google» | той самий                                              |
| `Empty` (порожня скринька)      | inline текст «Скринька порожня.»    | inline `<q-banner rounded dense>` нейтральним кольором |

Без `$q.notify` / `$q.dialog`. Помилки видно сторінкою, як зараз.

## Тести

**Vitest (vue-test-utils + mountWithQuasar):**

- Всі поточні тести у `Login.test.js` оновлюємо `mount` → `mountWithQuasar`. Текстові expectations не міняємо.
- Новий тест на skeleton: `inboxCount === null && inboxErrorKind === null` → `findComponent({ name: 'QSkeleton' })` exists.
- Помилка login → `findComponent({ name: 'QBanner' })` exists + текст збігається.
- Кнопка «Увійти через Google» — все ще `findAll('button').find(b => b.text().includes('Увійти через Google'))` працює.

**`auth-store.test.js`:** без змін — не торкає UI.

**`auth-errors.test.js`:** без змін.

**Smoke (manual):**

- `bun --cwd app run tauri dev` на macOS: візуальна перевірка картки, кнопок, dark mode toggle через System Settings.
- `bun --cwd app run android` (опційно): кнопка ripple + чіп виглядає material-3-outlined.

## Bundle вплив (приблизно)

- Quasar core: ~80–100 KB gzipped.
- Material Symbols Outlined font + CSS: ~150 KB.
- Загальний +~250 KB у `app/dist/`. Для Tauri (бінарник ~10 МБ) — ~2,5%, прийнятно.

## Ризики і мітігації

| Ризик                                                                        | Мітігація                                                                                                                                    |
| ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `@quasar/vite-plugin` конфлікт із `vue-macros`/`unplugin-auto-import`        | Тестуємо на dev-сервері одразу після setup; quasar-плагін офіційно сумісний з Vite 5/6/8 і кастомним vue плагіном через `transformAssetUrls` |
| Sass-залежність ламає білд на Android target                                 | `sass` тільки dev-dep, у бінарнику не опиняється; білд робиться один раз на CI                                                               |
| Material Symbols Outlined font ~150 КБ — забагато                            | Прийнятно, але якщо стане боляче — переходимо на iconify SVG-set окремою ітерацією                                                           |
| Тести з vue-test-utils не бачать Quasar-компонентів                          | `mountWithQuasar` helper + перевірити що `Quasar` plugin зареєстрований глобально для тестів                                                 |
| Dark mode зламає macOS-стиль                                                 | Quasar dark класи (`bg-dark`, `text-white`) автоматично; ручний тест на System Settings → Dark                                               |
| Tauri WKWebView macOS Mojave/Big Sur не має `-apple-system` для всіх weights | Fallback на `BlinkMacSystemFont, SF Pro Text, Inter, system-ui, sans-serif` уже покриває                                                     |
