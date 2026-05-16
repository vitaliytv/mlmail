# Quasar Refactor with macOS material-look Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ввести Quasar 2 як UI-фреймворк MLMaiL і переписати `Login.vue` на Quasar-компоненти з macOS material-look (system-ui font, macOS Accent Blue, мʼякший radius) без зміни UX-поведінки.

**Architecture:** Підключаємо Quasar через `@quasar/vite-plugin` у наявний Vite-конфіг, кастомний `quasar-variables.sass` для macOS-tuned defaults. `App.vue` обгортаємо в `<q-layout>`, `Login.vue` переписаний на `q-page`/`q-btn`/`q-card`/`q-chip`/`q-banner`/`q-skeleton`. Тести оновлюються через `mountWithQuasar` helper. Без `$q.dialog`/`$q.notify` — UX той самий.

**Tech Stack:** Vue 3, Vite 8, Quasar 2 + `@quasar/extras` (Material Symbols Outlined) + `@quasar/vite-plugin` + sass; Vitest + `@vue/test-utils`.

**Spec:** [docs/superpowers/specs/2026-05-16-quasar-refactor-design.md](../specs/2026-05-16-quasar-refactor-design.md)

---

## File Structure

**Setup-шар:**

- Modify: `app/package.json` — додати `quasar`, `@quasar/extras` (prod), `@quasar/vite-plugin`, `sass` (dev).
- Create: `app/src/quasar-variables.sass` — sass-defaults: macOS Accent Blue, system-ui font, button radius.
- Modify: `app/vite.config.js` — інтеграція `quasar` plugin, `transformAssetUrls` у vue plugin.
- Modify: `app/src/main.js` — реєстрація `Quasar` plugin + base CSS + Material Symbols Outlined.
- Modify: `app/src/App.vue` — обгортка `<q-layout>` + `<q-page-container>`, прибрати старе global CSS.

**Login-компонент:**

- Modify: `app/src/views/Login.vue` — переписаний на Quasar-компоненти.
- Modify: `app/src/views/Login.test.js` — `mount` → `mountWithQuasar`, новий тест на skeleton + q-banner.
- Create: `app/src/test-utils/quasar.js` — `mountWithQuasar` фабрика.

**Docs + cspell:**

- Modify: `docs/ci4/03-components.md` — Frontend UI Kit MLMaiL + Auth Component update.
- Modify: `docs/ci4/04-code.md` — нові секції: quasar-variables.sass, main.js, vite.config.js, App.vue.
- Modify: `docs/ci4/decisions.md` — рішення «UI = Quasar 2 + macOS material-look».
- Create: `docs/adr/_inbox/<ts>-quasar-ui.md` — ADR-нотатка.
- Modify (reactive): `.cspell.json` — нові українські слова з документації, якщо лінт скаже.

---

## Task 1: Add Quasar dependencies

**Files:**

- Modify: `app/package.json`

- [ ] **Step 1: Add deps and devDeps**

Edit `app/package.json`. Add to `dependencies` (alphabetical order doesn't matter; tests just check resolution):

```json
"@quasar/extras": "^1.16.12",
"quasar": "^2.17.5",
```

Add to `devDependencies`:

```json
"@quasar/vite-plugin": "^1.9.0",
"sass": "^1.81.0",
```

The full `app/package.json` after edits:

```json
{
  "name": "app",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "android": "tauri android dev",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "dependencies": {
    "@quasar/extras": "^1.16.12",
    "@tauri-apps/api": "^2.11.0",
    "quasar": "^2.17.5",
    "vue": "^3.5.34"
  },
  "devDependencies": {
    "@quasar/vite-plugin": "^1.9.0",
    "@tauri-apps/cli": "^2.11.1",
    "@vitejs/plugin-vue": "^6.0.6",
    "@vue/test-utils": "^2.4.6",
    "jsdom": "^25.0.1",
    "sass": "^1.81.0",
    "unplugin-auto-import": "^21.0.0",
    "vite": "^8.0.11",
    "vite-plugin-vue-layouts-next": "^1.0.6",
    "vitest": "^2.1.9",
    "vue-macros": "^3.1.2"
  },
  "engines": {
    "bun": ">=1.3",
    "node": ">=24"
  }
}
```

- [ ] **Step 2: Install**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail && bun install`
Expected: SUCCESS. Lockfile updated. `quasar`, `@quasar/extras`, `@quasar/vite-plugin`, `sass` resolved.

- [ ] **Step 3: Sanity check**

Run: `ls /Users/vitaliytv/www/vitaliytv/mlmail/node_modules/quasar/package.json /Users/vitaliytv/www/vitaliytv/mlmail/node_modules/@quasar/vite-plugin/package.json /Users/vitaliytv/www/vitaliytv/mlmail/node_modules/@quasar/extras/package.json /Users/vitaliytv/www/vitaliytv/mlmail/node_modules/sass/package.json`
Expected: всі 4 файли існують.

- [ ] **Step 4: Commit**

```bash
git add app/package.json bun.lock
git commit -m "$(cat <<'EOF'
feat(deps): add Quasar 2 + extras + vite-plugin + sass

Готує фундамент під рефакторинг Login.vue на Quasar-компоненти.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Create `quasar-variables.sass` with macOS material-look

**Files:**

- Create: `app/src/quasar-variables.sass`

- [ ] **Step 1: Write the file**

Create `app/src/quasar-variables.sass`:

```sass
// macOS material-look — sass overrides for Quasar defaults.
// Loaded by @quasar/vite-plugin (see vite.config.js).

$primary:   #0a84ff
$secondary: #5e5ce6
$accent:    #ff453a
$dark:      #1c1c1e
$dark-page: #000

$positive:  #30d158
$negative:  #ff453a
$info:      #64d2ff
$warning:   #ff9f0a

$typography-font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Inter', system-ui, sans-serif

$button-border-radius:  6px
$generic-border-radius: 8px
```

- [ ] **Step 2: Verify file**

Run: `cat app/src/quasar-variables.sass | head -20`
Expected: бачимо `$primary:   #0a84ff`.

(Поки що sass не процеситься — він буде підхоплений лише після Task 3.)

- [ ] **Step 3: Commit**

```bash
git add app/src/quasar-variables.sass
git commit -m "$(cat <<'EOF'
feat(quasar): sass-vars з macOS material-look

macOS Accent Blue для $primary, system-ui font stack з SF Pro,
мʼякший radius (6px кнопки, 8px generic). Файл підхоплюється
@quasar/vite-plugin у наступній задачі.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Wire `@quasar/vite-plugin` in `vite.config.js`

**Files:**

- Modify: `app/vite.config.js`

- [ ] **Step 1: Edit `vite.config.js`**

Replace the contents of `app/vite.config.js` with:

```js
import { quasar, transformAssetUrls } from '@quasar/vite-plugin'
import Vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import { defineConfig } from 'vite'
import Layouts from 'vite-plugin-vue-layouts-next'
import VueMacros from 'vue-macros/vite'

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    AutoImport({
      imports: ['vue', 'vue-router']
    }),
    VueMacros({
      plugins: {
        vue: Vue({ template: { transformAssetUrls } })
      }
    }),
    Layouts(),
    quasar({ sassVariables: 'src/quasar-variables.sass' })
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**']
    }
  },

  test: {
    environment: 'jsdom',
    globals: false,
    include: ['src/**/*.test.{js,vue}']
  }
}))
```

- [ ] **Step 2: Verify Vite can resolve plugin**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run build 2>&1 | tail -10`

Expected: build може фейлити на CSS/import якщо `main.js` ще не імпортує quasar — це нормально, головне щоб не падало на самому quasar-плагіні. Ловимо помилки виду «Cannot find module '@quasar/vite-plugin'» — їх НЕ має бути.

(Якщо є помилки про `src/main.js` — це означатиме що quasar-плагін загружено успішно, але код ще не повний. Це ОК і буде виправлено далі.)

- [ ] **Step 3: Commit**

```bash
git add app/vite.config.js
git commit -m "$(cat <<'EOF'
feat(quasar): wire @quasar/vite-plugin into vite.config.js

transformAssetUrls передаємо у vue plugin усередині VueMacros wrap.
sassVariables вказує на src/quasar-variables.sass з Task 2.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Register Quasar plugin in `main.js`

**Files:**

- Modify: `app/src/main.js`

- [ ] **Step 1: Read current `main.js`**

Run: `cat app/src/main.js`
Expected output:

```js
import App from './App.vue'

createApp(App).mount('#app')
```

(`createApp` auto-imported via `unplugin-auto-import`.)

- [ ] **Step 2: Update `main.js`**

Replace `app/src/main.js` contents with:

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

- [ ] **Step 3: Sanity build**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run build 2>&1 | tail -20`
Expected: SUCCESS, або помилки тільки в `App.vue`/`Login.vue` (вони ще не торкнуті). Quasar CSS повинен підхопитись.

- [ ] **Step 4: Commit**

```bash
git add app/src/main.js
git commit -m "$(cat <<'EOF'
feat(quasar): register Quasar plugin in main.js

iconSet=material-symbols-outlined, config.dark=auto (наслідує OS).
Імпортуємо quasar base CSS і material-symbols-outlined font/CSS.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Wrap `App.vue` in `<q-layout>`

**Files:**

- Modify: `app/src/App.vue`

- [ ] **Step 1: Replace contents**

Overwrite `app/src/App.vue`:

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

Старі `<style>` блоки прибираємо повністю — Quasar бере dark mode і базовий typography на себе.

- [ ] **Step 2: Sanity build**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run build 2>&1 | tail -10`
Expected: SUCCESS (Login.vue ще не оновлений, але має лишитися сумісним).

- [ ] **Step 3: Commit**

```bash
git add app/src/App.vue
git commit -m "$(cat <<'EOF'
feat(app): wrap App.vue in q-layout/q-page-container

Прибирає global :root + prefers-color-scheme — це тепер на Quasar
через config.dark='auto'.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Create `mountWithQuasar` test helper (TDD prep)

**Files:**

- Create: `app/src/test-utils/quasar.js`

- [ ] **Step 1: Write helper**

Create `app/src/test-utils/quasar.js`:

```js
import { mount } from '@vue/test-utils'
import { Quasar } from 'quasar'

/**
 * @param {object} component Vue component
 * @param {object} [options] mount options (forwarded)
 * @returns {object} test wrapper
 */
export function mountWithQuasar(component, options = {}) {
  const userPlugins = (options.global && options.global.plugins) || []
  return mount(component, {
    ...options,
    global: {
      ...(options.global || {}),
      plugins: [
        ...userPlugins,
        [Quasar, { config: { dark: false } }]
      ]
    }
  })
}
```

- [ ] **Step 2: Quick verify**

Run: `node --input-type=module -e "import('/Users/vitaliytv/www/vitaliytv/mlmail/app/src/test-utils/quasar.js').then(m => console.log(typeof m.mountWithQuasar))"`
Expected: `function` (підтверджує синтаксис ОК).

- [ ] **Step 3: Commit**

```bash
git add app/src/test-utils/quasar.js
git commit -m "$(cat <<'EOF'
test: add mountWithQuasar helper

Спільна фабрика для тестів Vue-компонентів, що потребують Quasar
plugin зареєстрованим. Виставляє dark=false щоб тести були
детерміністичні незалежно від OS-prefers.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Refactor `Login.vue` to Quasar components

**Files:**

- Modify: `app/src/views/Login.vue`

- [ ] **Step 1: Rewrite the component**

Overwrite `app/src/views/Login.vue` with:

```vue
<script setup>
import { useAuthStore } from '../services/auth-store.js'
import { errorMessage } from '../i18n/auth-errors.js'

const auth = useAuthStore()
onMounted(() => auth.initialize())
</script>

<template>
  <q-page class="flex flex-center column q-gutter-md q-pa-md">
    <div class="text-h5 q-mb-md">MLMaiL</div>

    <template v-if="auth.isAuthenticated.value">
      <div class="text-body1">Ви увійшли як {{ auth.email.value }}</div>

      <q-chip
        v-if="auth.inboxCount.value !== null"
        icon="mail"
        color="primary"
        text-color="white"
      >
        Листів у скриньці: {{ auth.inboxCount.value }}
      </q-chip>
      <q-banner
        v-else-if="auth.inboxErrorKind.value"
        class="bg-red-1 text-red-9"
        rounded
        dense
      >
        {{ errorMessage(auth.inboxErrorKind.value) }}
      </q-banner>
      <q-skeleton v-else type="QChip" width="180px" />

      <q-card v-if="auth.currentMessage.value" flat bordered class="message-card">
        <q-card-section>
          <div><strong>Від:</strong> {{ auth.currentMessage.value.from }}</div>
          <div><strong>Тема:</strong> {{ auth.currentMessage.value.subject }}</div>
          <div><strong>Дата:</strong> {{ auth.currentMessage.value.date }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section>
          <pre class="message-body">{{ auth.currentMessage.value.body }}</pre>
        </q-card-section>
      </q-card>
      <div v-else-if="auth.isMessageLoading.value" class="message-card column q-gutter-sm">
        <q-skeleton type="text" width="60%" />
        <q-skeleton type="text" width="40%" />
        <q-skeleton type="text" width="50%" />
        <q-skeleton type="rect" height="80px" />
      </div>
      <q-banner
        v-else-if="auth.messageErrorKind.value"
        class="bg-red-1 text-red-9"
        rounded
        dense
      >
        {{ errorMessage(auth.messageErrorKind.value) }}
      </q-banner>

      <div class="row q-gutter-sm">
        <q-btn
          color="primary"
          icon="refresh"
          :loading="auth.isMessageLoading.value"
          @click="auth.loadRandomMessage()"
        >
          Показати інший
        </q-btn>
        <q-btn flat color="grey-8" icon="logout" @click="auth.logout()">
          Вийти
        </q-btn>
      </div>
    </template>

    <q-btn
      v-else
      color="primary"
      icon-right="login"
      :loading="auth.isLoading.value"
      @click="auth.login()"
    >
      Увійти через Google
      <template #loading>Зачекайте…</template>
    </q-btn>

    <q-banner
      v-if="auth.errorKind.value"
      class="bg-red-1 text-red-9"
      rounded
      dense
    >
      {{ errorMessage(auth.errorKind.value) }}
    </q-banner>
  </q-page>
</template>

<style scoped>
.message-card {
  max-width: 60ch;
  width: 100%;
}

.message-body {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  margin: 0;
  font-family: inherit;
}
</style>
```

- [ ] **Step 2: Dev sanity (optional — skip if no browser session)**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run build 2>&1 | tail -10`
Expected: SUCCESS without warnings про невідомі компоненти `q-page`/`q-btn`/etc.

- [ ] **Step 3: Commit**

```bash
git add app/src/views/Login.vue
git commit -m "$(cat <<'EOF'
feat(login): rewrite Login.vue on Quasar components

q-page для контейнера, q-btn з вбудованим loading-станом, q-chip
для inbox count, q-card для випадкового листа, q-banner для inline
помилок, q-skeleton як placeholder для loading. UX той самий —
logout одразу, помилки inline, без $q.dialog/$q.notify.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Update `Login.test.js` — `mount` → `mountWithQuasar` + новий q-skeleton тест

**Files:**

- Modify: `app/src/views/Login.test.js`

- [ ] **Step 1: Replace imports and rewrite tests**

Open `app/src/views/Login.test.js`. Замінити верхню секцію (рядки 1-14) на:

```js
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { mountWithQuasar } from '../test-utils/quasar.js'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { _resetForTest } = await import('../services/auth-store.js')
const loginModule = await import('./Login.vue')
const Login = loginModule.default

beforeEach(() => {
  invokeMock.mockReset()
  _resetForTest()
})
```

Далі в усьому файлі replace-all `mount(Login)` → `mountWithQuasar(Login)`. Решта тестів не міняється — текстові assertions працюють як були.

- [ ] **Step 2: Append a skeleton test**

В кінець файлу (після останнього `})` блоку `describe('Login.vue random message'`) додати:

```js
describe('Login.vue Quasar UX', () => {
  it('shows QSkeleton instead of inbox count while loading', async () => {
    let resolveCount
    // oxlint-disable-next-line promise/avoid-new
    const pending = new Promise(resolve => { resolveCount = resolve })
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return pending
      return Promise.resolve(null)
    })
    const w = mountWithQuasar(Login)
    await flushPromises()
    const skeletons = w.findAllComponents({ name: 'QSkeleton' })
    expect(skeletons.length).toBeGreaterThan(0)
    resolveCount(5)
    await flushPromises()
  })

  it('renders QBanner with red classes on inbox error', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count')
        return Promise.reject(Object.assign(new Error('Http'), { kind: 'Http' }))
      return Promise.resolve(null)
    })
    const w = mountWithQuasar(Login)
    await flushPromises()
    const banner = w.findComponent({ name: 'QBanner' })
    expect(banner.exists()).toBe(true)
    expect(banner.text()).toContain('Gmail повернув помилку. Спробуйте пізніше.')
  })
})
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run test 2>&1 | tail -15`
Expected: усі тести PASS (попередні Login.vue 11 + 2 нові = 13; auth-store 23; auth-errors 11; разом ~47).

Якщо `q-btn` не рендериться як `<button>` для тестів типу `findAll('button').find(...)` — це означає що Quasar глобальні компоненти не зареєстровані. Перевірити, що `mountWithQuasar` встановлює `Quasar` plugin. Зазвичай це працює з коробки.

- [ ] **Step 4: Commit**

```bash
git add app/src/views/Login.test.js
git commit -m "$(cat <<'EOF'
test(login): mount → mountWithQuasar + skeleton/banner specs

Усі тести Login.vue тепер використовують mountWithQuasar helper.
Додано два нових тести: QSkeleton при loading inbox count, QBanner
з червоними класами при Gmail-помилці.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Manual lint pass and cspell additions

**Files:**

- Modify (reactively): `.cspell.json`

- [ ] **Step 1: Run lint**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail && bun run lint 2>&1 | tail -30`
Expected: можливі `cspell` помилки на нові слова (наприклад «таблицю», «обгортка», «defaults», «iconset», «outline»). Зібрати список усіх «Unknown word» рядків.

Якщо є інші помилки (oxlint/eslint/stylelint/markdownlint) — виправити їх інлайн відповідно до повідомлень.

- [ ] **Step 2: Add missing words to `.cspell.json`**

Read current `.cspell.json`, append unique missing words (українські) в `words` array, наприклад:

```json
    "tauri-pipeline",
    "frontmatter",
    "iconset"
```

(точний список залежить від виводу — додавай лише ті слова, які реально показав cspell).

- [ ] **Step 3: Re-run lint**

Run: `bun run lint 2>&1 | tail -10; echo "exit=$?"`
Expected: `oxlint`, `eslint`, `stylelint`, `markdownlint`, `cspell` усі чисті. (Pre-existing `run-shellcheck-text.mjs` script error — не від цього рефакторингу; ігноруємо.)

- [ ] **Step 4: Run vitest one more time to make sure nothing broke**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run test 2>&1 | tail -5`
Expected: усі PASS.

- [ ] **Step 5: Commit (if .cspell.json changed)**

```bash
git add .cspell.json
git commit -m "$(cat <<'EOF'
chore(cspell): add words introduced by Quasar refactor docs

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Update `docs/ci4/03-components.md`

**Files:**

- Modify: `docs/ci4/03-components.md`

- [ ] **Step 1: Add Frontend UI Kit MLMaiL block to the diagram and text**

У Mermaid діаграмі (`graph TB` для контейнера MLMaiL Frontend) додати новий вузол перед `AuthStore`:

```
    UIKit[Frontend UI Kit MLMaiL<br/>Quasar 2 + macOS material-look<br/>implemented]
```

І стрілки:

```
    Auth --> UIKit
    UIKit --> AppShell
```

Після секції «Компонент App Shell MLMaiL» додати:

```markdown
### Компонент Frontend UI Kit MLMaiL (implemented)

Frontend UI Kit MLMaiL — Quasar 2 як UI-фреймворк MLMaiL. Підключений через
[`@quasar/vite-plugin`](../../app/vite.config.js) з кастомним
[quasar-variables.sass](../../app/src/quasar-variables.sass) (macOS Accent
Blue для `$primary`, system-ui font stack із SF Pro, мʼякший radius:
кнопки 6px, generic 8px). Глобально зареєстрований у
[main.js](../../app/src/main.js) із `iconSet: 'material-symbols-outlined'`
та `config.dark: 'auto'` (наслідує OS prefers-color-scheme).

Усі Vue-компоненти MLMaiL використовують Quasar-компоненти замість
сирого HTML: `q-page`, `q-btn`, `q-card`, `q-chip`, `q-banner`, `q-skeleton`,
`q-separator`, `q-layout`, `q-page-container`. Іконки — Material Symbols
Outlined через `@quasar/extras`.

Тести Vue-компонентів MLMaiL використовують
[mountWithQuasar](../../app/src/test-utils/quasar.js) фабрику — обгортку
над `mount()` з Vue Test Utils, яка глобально реєструє Quasar plugin із
`dark: false` (детерміністично).
```

- [ ] **Step 2: Update Auth Component description**

У секції «Компонент Auth Component MLMaiL» замінити рядок «Залежить від: Auth Store MLMaiL, Auth Errors i18n MLMaiL.» на:

```
Залежить від: Auth Store MLMaiL, Auth Errors i18n MLMaiL, Frontend UI Kit MLMaiL (Quasar).
```

І в описі UI-гілок додати примітку про Quasar-компоненти:

Замінити список (`- не авторизовано → ...`) на:

```
- не авторизовано → `q-btn` "Увійти через Google" (вбудований loading-стан,
  текст у `#loading` слоті — "Зачекайте…");
- авторизовано → "Ви увійшли як {email}", `q-chip` "Листів у скриньці: N"
  (або `q-skeleton` поки число вантажиться; `q-banner` при помилці Gmail),
  `q-card` з випадковим листом (Від/Тема/Дата у `q-card-section`, `<pre>`
  тіло) або `q-skeleton`-рядки під час завантаження; `q-btn` "Показати
  інший" із loading-станом і `q-btn` "Вийти";
- помилка останньої спроби логіну — `q-banner` з українським рядком
  з Auth Errors i18n MLMaiL.
```

- [ ] **Step 3: Commit**

```bash
git add docs/ci4/03-components.md
git commit -m "$(cat <<'EOF'
docs(ci4): components — додати Frontend UI Kit MLMaiL (Quasar)

Auth Component MLMaiL тепер залежить від Quasar; список Quasar-компонентів
у UI-гілках Login.vue зафіксовано.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Update `docs/ci4/04-code.md`

**Files:**

- Modify: `docs/ci4/04-code.md`

- [ ] **Step 1: Add `quasar-variables.sass` section**

Після блоку «Файл [app/src/i18n/auth-errors.js]» (десь близько до існуючої секції про auth-store), додати:

```markdown
### Файл [app/src/quasar-variables.sass](../../app/src/quasar-variables.sass)

Sass-таблиця, що перевизначає Quasar-defaults для macOS material-look:
`$primary: #0a84ff` (macOS Accent Blue), `$typography-font-family:
-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Inter', system-ui,
sans-serif`, `$button-border-radius: 6px`, `$generic-border-radius: 8px`.
Підхоплюється через `@quasar/vite-plugin` (див.
[vite.config.js](../../app/vite.config.js) — поле `sassVariables`).

### Файл [app/src/test-utils/quasar.js](../../app/src/test-utils/quasar.js)

Test helper `mountWithQuasar(component, options)` — обгортка над `mount()`
з `@vue/test-utils`, яка глобально реєструє Quasar plugin із
`dark: false`. Використовується усіма Vue-component тестами, щоб
Quasar-компоненти коректно резолвилися.
```

- [ ] **Step 2: Update `main.js` section**

Знайти секцію «Файл [app/src/main.js]». Замінити поточний код-блок на:

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

І текст під ним — на:

```
Точка входу MLMaiL Frontend Bootstrap. Реєструє Quasar 2 як plugin із
Material Symbols Outlined іконками та dark mode = 'auto' (наслідує
OS prefers-color-scheme). `createApp` доступний без імпорту завдяки
`unplugin-auto-import`. Quasar bootstraps базовий CSS та шрифт-набір.
```

- [ ] **Step 3: Update `vite.config.js` section**

Знайти секцію «Файл [app/vite.config.js]». Замінити перелік ключових рішень — додати рядок:

```
- Quasar через `@quasar/vite-plugin` з `sassVariables: 'src/quasar-variables.sass'`,
  `transformAssetUrls` передається у `vue` plugin усередині `VueMacros` wrap.
```

- [ ] **Step 4: Update `App.vue` section**

Замінити приклад коду на новий:

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

І текст під ним — на:

```
Кореневий компонент App Shell MLMaiL. Тонка обгортка над `<q-layout>` +
`<q-page-container>` — Quasar bootstrap layout (без drawer/header/footer
поки). Рендерить `<Login />` напряму. Global CSS / dark mode — на боці
Quasar (`config.dark: 'auto'`).
```

- [ ] **Step 5: Update `Login.vue` section**

Замінити опис на:

```
Auth Component MLMaiL — Vue-компонент Login-екрану. Переписаний на
Quasar-компоненти (`q-page`, `q-btn`, `q-card`, `q-chip`, `q-banner`,
`q-skeleton`, `q-separator`). Дві гілки шаблону (авторизований / не
авторизований), обидві українською. Підключає Auth Store MLMaiL і Auth
Errors i18n MLMaiL. Тестується у
[app/src/views/Login.test.js](../../app/src/views/Login.test.js) через
Vitest + [mountWithQuasar](../../app/src/test-utils/quasar.js).
```

- [ ] **Step 6: Commit**

```bash
git add docs/ci4/04-code.md
git commit -m "$(cat <<'EOF'
docs(ci4): code-level — quasar-variables, main.js, vite.config, App.vue, Login.vue

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Update `decisions.md` + ADR-inbox note

**Files:**

- Modify: `docs/ci4/decisions.md`
- Create: `docs/adr/_inbox/<ts>-quasar-ui.md`

- [ ] **Step 1: Append decision to `decisions.md`**

Перед секцією «## Рішення, що очікують ADR для MLMaiL», після останнього прийнятого рішення (про random message), додати:

```markdown
### Рішення: UI-фреймворк MLMaiL — Quasar 2 з macOS material-look

Закодовано у [app/package.json](../../app/package.json),
[app/vite.config.js](../../app/vite.config.js),
[app/src/main.js](../../app/src/main.js),
[app/src/quasar-variables.sass](../../app/src/quasar-variables.sass),
[app/src/App.vue](../../app/src/App.vue),
[app/src/views/Login.vue](../../app/src/views/Login.vue).

Quasar 2 підключається через `@quasar/vite-plugin` у наявний Vite-конфіг
(не Quasar CLI), що зберігає Tauri-pipeline і всі попередні плагіни.
Кастомний `quasar-variables.sass` дає macOS material-look: macOS Accent
Blue (`#0a84ff`) для `$primary`, system-ui font stack із SF Pro, мʼякший
radius (6px кнопки, 8px generic). Іконки — Material Symbols Outlined
(`@quasar/extras`). Dark mode — `auto` за OS prefers-color-scheme.

Узгоджується з `.cursor/rules/vue.mdc` («Використовуй Quasar для
компонентів»). Tauri-таргети macOS та Android, без вебу, без Windows;
Quasar mobile-first + платформ-aware дає кращий out-of-the-box UX, ніж
plain HTML або Reka UI.

Вплив на C4-модель MLMaiL:

- [03-components.md](03-components.md) — додано Frontend UI Kit MLMaiL
  (Quasar) як новий компонент, Auth Component MLMaiL переключений на
  Quasar-компоненти, mountWithQuasar test-helper зафіксовано.
- [04-code.md](04-code.md) — нові секції для `quasar-variables.sass`,
  `test-utils/quasar.js`, оновлено `main.js`, `vite.config.js`, `App.vue`,
  `Login.vue`.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0009-quasar-ui.md`.
Чернетка інбоксу — у `docs/adr/_inbox/`.
```

- [ ] **Step 2: Generate timestamp and write ADR-inbox**

Run: `date +%Y%m%d-%H%M%S`
Скопіюй вивід як `<TS>`.

Create `docs/adr/_inbox/<TS>-quasar-ui.md`:

```markdown
---
session: brainstorm
captured: 2026-05-16
---

## UI-фреймворк MLMaiL: Quasar 2 з macOS material-look

**Контекст:** MLMaiL — Tauri-застосунок на macOS + Android. У `.cursor/rules/vue.mdc` зафіксовано правило «Використовуй Quasar для компонентів». До цієї ітерації UI був на сирому HTML/CSS.

**Рішення/Процедура/Факт:** Quasar 2 підключаємо через `@quasar/vite-plugin` (не Quasar CLI). `quasar-variables.sass` дає macOS material-look: `$primary: #0a84ff` (macOS Accent Blue), system-ui font stack з SF Pro, button radius 6px, generic radius 8px. Іконки — Material Symbols Outlined через `@quasar/extras`. Dark mode — `auto` за OS. `Login.vue` переписаний на `q-page`/`q-btn`/`q-card`/`q-chip`/`q-banner`/`q-skeleton`. Тести через `mountWithQuasar` helper.

**Обґрунтування:** Material = native UX на Android (60% користувацької бази). macOS material-tuned варіант (SF Pro font, мʼякший radius, macOS Accent Blue) виглядає прийнятно і там. Quasar дає готові q-skeleton/q-banner/q-card/q-chip, які точно знадобляться у наступних ітераціях (списки листів, AI-саммері, нотатки). Bundle ~250 КБ (Quasar core + Material Symbols Outlined) — несуттєво для Tauri-бінарника. UX-поведінка цієї ітерації не змінюється — лише візуальна заміна + q-skeleton як loading-патерн.

**Розглянуті альтернативи:**

- Plain HTML + ручний CSS — швидко, але всі UX-патерни (touch ripple, safe-area, swipe-dismiss) пишемо самі. Не задовольняє правило.
- Reka UI (Radix Vue) — headless примітиви + Tailwind/власні стилі. Дає design freedom, але немає material look для Android — більше роботи.
- Quasar CLI замість Vite-plugin — переписав би весь pipeline, втратив би Tauri-сумісність із існуючими плагінами (`unplugin-auto-import`, `vue-macros`, `vite-plugin-vue-layouts-next`). Відкинуто.

**Зачіпає:** `app/package.json`, `app/vite.config.js`, `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`, `app/src/views/Login.test.js`, `app/src/quasar-variables.sass` (новий), `app/src/test-utils/quasar.js` (новий), `docs/ci4/*`, `.cursor/rules/vue.mdc` (як правило, що тепер задовольняється).
```

- [ ] **Step 3: Commit**

```bash
git add docs/ci4/decisions.md docs/adr/_inbox/
git commit -m "$(cat <<'EOF'
docs: ADR-inbox + decisions для Quasar рішення

Фіксує вибір Quasar 2 + macOS material-look у C4-моделі та ADR-inbox.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 13: Final verification

- [ ] **Step 1: Full vitest run**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run test 2>&1 | tail -10`
Expected: усі тести PASS (existing 45 + 2 нових Quasar = 47).

- [ ] **Step 2: Vite production build**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app && bun run build 2>&1 | tail -15`
Expected: SUCCESS, без warnings про unresolved Quasar-компоненти. У `dist/` зʼявиться `index.html` + assets.

- [ ] **Step 3: Repo-wide lint**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail && bun run lint 2>&1 | tail -5; echo "exit=$?"`
Expected: `oxlint`/`eslint`/`jscpd`/`cspell`/`markdownlint`/`stylelint` чисті. (Pre-existing `run-shellcheck-text.mjs` warning ігноруємо.)

- [ ] **Step 4: Rust tests (regression sanity, бо ми не торкали Rust)**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail/app/src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: усі PASS (64 тести), без змін.

- [ ] **Step 5: Smoke checklist (manual; виконує користувач)**

1. `bun --cwd app run tauri dev` на macOS — побачити нову візуальну стилістику Login.vue: q-chip із синім macOS Accent для inbox count, q-card для листа, q-btn із refresh-іконкою для «Показати інший», q-banner для помилок.
2. Перемкнути System Settings → Appearance → Dark — UI має автоматично перейти у dark mode.
3. Натиснути «Показати інший» — побачити q-skeleton-рядки під час завантаження, потім нову картку.
4. Розлогінитися → знову на «Увійти через Google» (без confirm-діалогу, як зараз).
5. (Опційно) `bun --cwd app run android` — на Android-емуляторі побачити Material-look з touch ripple на q-btn.

---

## Self-Review

**Spec coverage:**

| Spec requirement | Covered by |
| --- | --- |
| Quasar deps + sass | Task 1 |
| `quasar-variables.sass` із macOS-look | Task 2 |
| `@quasar/vite-plugin` integration | Task 3 |
| `main.js` Quasar registration | Task 4 |
| `App.vue` → `q-layout` | Task 5 |
| `mountWithQuasar` helper | Task 6 |
| `Login.vue` Quasar rewrite | Task 7 |
| `Login.test.js` адаптація + новий q-skeleton тест | Task 8 |
| Material Symbols Outlined іконки | Task 4 (CSS imports) |
| Dark mode auto | Task 4 (config.dark) |
| UX без зміни (logout одразу, inline banners) | Task 7 (немає $q.dialog/$q.notify) |
| `auth-store` без змін | (нічого не торкає) |
| `auth-store.test.js` без змін | (нічого не торкає) |
| `auth-errors.js` без змін | (нічого не торкає) |
| docs 03-components.md | Task 10 |
| docs 04-code.md | Task 11 |
| docs decisions.md + ADR-inbox | Task 12 |
| Поза scope (drawer, $q.notify, pull-to-refresh) | не реалізуємо (свідомо) |
| `.cspell.json` оновлення реактивно | Task 9 |

Прогалин не знайдено.

**Placeholder scan:** Усі тестові тіла повні, `<TS>` явно генерується через `date +%Y%m%d-%H%M%S` у Task 12 Step 2, версії пакетів — конкретні.

**Type consistency:**

- `mountWithQuasar` (не `mountQuasar`/`mountInQuasar`) — стабільно у Task 6, 8.
- `Quasar` plugin — імпорт зі `'quasar'` стабільно скрізь.
- `quasar-variables.sass` шлях — `src/quasar-variables.sass` від кореня `app/` (так і у `vite.config.js`).
- `iconSet: 'material-symbols-outlined'` — те саме у `main.js` і у CSS-імпорті `@quasar/extras/material-symbols-outlined/material-symbols-outlined.css`.
- `dark: 'auto'` у `main.js`, `dark: false` у `mountWithQuasar` — навмисно різні (test detrminism), задокументовано у спеці.

Все консистентно.
