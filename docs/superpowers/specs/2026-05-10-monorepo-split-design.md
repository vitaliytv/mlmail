# Розділення проєкту на монорепо: корінь + workspace `app`

**Дата:** 2026-05-10
**Статус:** затверджено для імплементації

## Мета

Розділити поточний одношаровий Tauri + Vue + Vite проєкт на bun-монорепо з одним workspace під назвою `app/`, у який переїжджає увесь код, що стосується Tauri-додатку (frontend + Rust). Корінь стає workspace-root, утримує тільки конфіги, документацію й shared dev-tooling.

## Цільова структура

```
mlmail/
├─ app/                          # workspace «app»
│  ├─ src/                       (← з кореня)
│  ├─ src-tauri/                 (← з кореня)
│  ├─ public/                    (← з кореня)
│  ├─ index.html                 (← з кореня)
│  ├─ vite.config.js             (← з кореня)
│  └─ package.json               (новий)
├─ .claude/  .cursor/  .github/  (без змін)
├─ .n-cursor.json                (без змін)
├─ .oxlintrc.json                (без змін)
├─ bunfig.toml                   (без змін)
├─ eslint.config.js              (правка двох рядків)
├─ AGENTS.md  CLAUDE.md  README.md  (без змін)
├─ docs/                         (нова тека для spec-ів)
├─ package.json                  (переписаний)
└─ bun.lock                      (regenerated)
```

## Розподіл `package.json`

### Кореневий `package.json`

```json
{
  "name": "mlmail",
  "private": true,
  "version": "0.1.0",
  "workspaces": ["app"],
  "devDependencies": {
    "@nitra/cursor": "^1.8.221"
  }
}
```

- Без скриптів `dev`/`build`/`preview`/`tauri` — користувач вирішив **не** робити proxy-скриптів. Запуск через `cd app && bun run <script>` або `bun --cwd app run <script>`.
- `@nitra/cursor` залишається у корені, бо обслуговує весь репо (rules/skills у `.cursor/`, `.claude/`).

### `app/package.json` (новий)

```json
{
  "name": "app",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.11.0",
    "@tauri-apps/plugin-opener": "^2.5.4",
    "vue": "^3.5.34"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.11.1",
    "@vitejs/plugin-vue": "^6.0.6",
    "vite": "^8.0.11"
  }
}
```

Версії — поточні (вже оновлені через попередній прогін `n-taze`).

## Правки конфігів

### `eslint.config.js`

Два рядки залежать від шляхів:

```diff
 import { getConfig } from '@nitra/eslint-config'

 export default [
   {
-    ignores: ['**/auto-imports.d.ts', 'src-tauri/**', 'dist/**']
+    ignores: ['**/auto-imports.d.ts', 'app/src-tauri/**', 'app/dist/**']
   },
   ...getConfig({
-    vue: ['.']
+    vue: ['app']
   })
 ]
```

Шляхи специфічні (`app/...`), не універсальні `**/...` — точніше відображає поточну структуру.

### Без змін

- `src-tauri/tauri.conf.json` — `frontendDist: "../dist"` відносно `src-tauri/`. Після переїзду розв'язується в `app/dist`, що збігається з vite default outDir відносно `app/`.
- `src-tauri/tauri.conf.json` — `beforeDevCommand`/`beforeBuildCommand` виконуються з CWD виклику `tauri`; виклик буде з `app/`, де `bun run dev` / `bun run build` існують.
- `src-tauri/Cargo.toml` — Rust-крейт `mlmail` / lib `mlmail_lib` не пов’язаний із workspace-name.
- `.oxlintrc.json` — `ignorePatterns` уже з `**/`-префіксом.
- `bunfig.toml` — глобальний для репо.
- `.n-cursor.json` — без шляхових пресетів.

## Послідовність переїзду

Послідовність важлива для атомарності перевірки.

1. **Видалити деривативи зі старого CWD:** `node_modules/`, `dist/`, `src-tauri/target/` (інакше після переїзду конфіги вкажуть у мертві місця).
2. **Створити `app/`** і перенести в нього: `src/`, `src-tauri/`, `public/`, `index.html`, `vite.config.js`. Через `git mv`, щоб історія Git зберіглась.
3. **Записати** `app/package.json` із вмістом вище.
4. **Переписати** кореневий `package.json` із вмістом вище. Видалити з нього всі app-залежності й app-скрипти.
5. **Оновити** `eslint.config.js` згідно з diff.
6. **`bun install`** із кореня — генерує нову `bun.lock` із workspace-резолвом.
7. **Верифікація:**
   - `bun --cwd app run build` — Vite + plugin-vue знаходять `index.html`/`src/`.
   - `cargo check --manifest-path app/src-tauri/Cargo.toml` — Rust-крейт компілюється.
8. **Видалити** перенесені файли з кореня (якщо `git mv` не зробив це автоматично — зазвичай робить).

## Що **не** входить у скоуп

- Не змінюємо `productName`, `identifier`, `version`, назву Rust-крейту.
- Не додаємо нові workspace-и (`packages/`, `shared/` тощо). Якщо знадобляться — окремий spec.
- Не вмикаємо proxy-скрипти в корені.
- Не міняємо `@nitra/cursor` rules/skills — вони залишаються кореневими.
- Не зачіпаємо CI (`.github/`) — поточні дії або працюватимуть із `bun --cwd app run build`, або потребуватимуть окремого PR.

## Верифікація після імплементації

Команди з кореня (всі мають дати exit 0):

```bash
bun install
bun --cwd app run build
cargo check --manifest-path app/src-tauri/Cargo.toml
```

Артефакти, які мають з’явитися: `app/dist/index.html`, `app/dist/assets/*`. `app/src-tauri/target/debug/...` після `cargo check`.

## Ризики й мітигації

- **Ризик:** забути перенести прихований файл (наприклад, `app/.gitignore`-подібний). **Мітигація:** після переїзду — `git status` має показати тільки очікувані `R` (renames) для перенесених, без `M` чи `??` для решти кореневих файлів.
- **Ризик:** `eslint.config.js` зламається через невідому семантику опції `vue: ['app']` у `@nitra/eslint-config`. **Мітигація:** запустити будь-який доступний lint-сценарій (наприклад, `bunx eslint app/src --max-warnings=0`) одразу після правки конфіга й до commit.
- **Ризик:** Tauri в новому розташуванні не знаходить ікон у `bundle.icon: ["icons/32x32.png", ...]` (відносні шляхи в `tauri.conf.json`). **Мітигація:** перевірив — шляхи відносні до `tauri.conf.json`, тобто до `app/src-tauri/`, тож `icons/...` ведуть у `app/src-tauri/icons/`, що збігається з фактом.
