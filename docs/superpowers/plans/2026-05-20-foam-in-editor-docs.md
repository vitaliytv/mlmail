# Foam In-Editor Docs Navigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Інженер навігує і читає C4/ADR-документацію MLMaiL всередині VS Code — з відрендереними Mermaid-діаграмами, переходом `Cmd+Click` по лінках і backlinks — без змін у docs:regen.

**Architecture:** Чисто конфігураційна зміна — три файли, нуль коду. Два VS Code-розширення (`foam.foam-vscode`, `bierner.markdown-mermaid`) додаються як workspace-рекомендації; `foam.files.ignore` у `.vscode/settings.json` тримає індекс Foam чистим; `docs/ci4/README.md` отримує секцію-інструкцію для інженера. Foam пасивно будує граф/backlinks з наявних markdown-лінків (sentinel-мітки ADR, `decisions.md`).

**Tech Stack:** VS Code workspace config (JSON), Markdown. Розширення: Foam (`foam.foam-vscode`), Markdown Preview Mermaid Support (`bierner.markdown-mermaid`).

**Spec:** [docs/superpowers/specs/2026-05-20-foam-graph-design.md](../specs/2026-05-20-foam-graph-design.md)

---

## File Structure

- Modify: `.vscode/extensions.json` — додати два ID у масив `recommendations`.
- Modify: `.vscode/settings.json` — додати один ключ `foam.files.ignore`.
- Modify: `docs/ci4/README.md` — додати секцію «Читання документації MLMaiL у VS Code».

Нових файлів немає. Коду немає. Автотестів немає — перевірка ручна (конфігурація редактора).

**Увага:** на момент написання плану `.vscode/extensions.json` має staged-модифікацію від стороннього процесу. Перед роботою — звірити поточний вміст файлу через Read, редагувати від актуального стану, не покладатися на лістинг нижче як на істину.

---

## Task 1: Додати Foam і Mermaid у рекомендовані розширення

**Files:**

- Modify: `.vscode/extensions.json`

**Why:** VS Code при відкритті репозиторію MLMaiL пропонує встановити розширення зі списку `recommendations`. Foam дає backlinks і навігацію, `bierner.markdown-mermaid` рендерить Mermaid-діаграми у preview.

- [ ] **Step 1: Прочитати поточний `.vscode/extensions.json`**

Run: відкрити `.vscode/extensions.json` через Read.
Expected: JSON з масивами `recommendations` і `unwantedRecommendations`. Очікуваний поточний `recommendations` (звірити фактичний):

```json
"recommendations": [
  "dbaeumer.vscode-eslint",
  "github.vscode-github-actions",
  "oxc.oxc-vscode",
  "DavidAnson.vscode-markdownlint",
  "timonwong.shellcheck",
  "redhat.vscode-yaml",
  "irongeek.vscode-env",
  "stylelint.vscode-stylelint",
  "Vue.volar",
  "tauri-apps.tauri-vscode",
  "rust-lang.rust-analyzer"
]
```

- [ ] **Step 2: Додати два ID після `DavidAnson.vscode-markdownlint`**

У масиві `recommendations` замінити рядок:

```json
  "DavidAnson.vscode-markdownlint",
```

на:

```json
  "DavidAnson.vscode-markdownlint",
  "foam.foam-vscode",
  "bierner.markdown-mermaid",
```

Це групує markdown-тулінг разом. `unwantedRecommendations` не чіпати.

- [ ] **Step 3: Перевірити, що JSON валідний**

Run: `bun -e "JSON.parse(await Bun.file('.vscode/extensions.json').text()); console.log('valid')"`
Expected: `valid`.

- [ ] **Step 4: Перевірити, що обидва ID присутні**

Run: `grep -E 'foam.foam-vscode|bierner.markdown-mermaid' .vscode/extensions.json`
Expected: два рядки знайдено.

- [ ] **Step 5: `git status && git diff .vscode/extensions.json` і зупинка**

Run: `git status && git diff .vscode/extensions.json`
Очікувано: у `recommendations` додано рівно два рядки. Зупинись, дай юзеру переглянути.

---

## Task 2: Додати `foam.files.ignore` у налаштування воркспейсу

**Files:**

- Modify: `.vscode/settings.json`

**Why:** Foam індексує весь markdown у воркспейсі. Без ignore-списку в граф і backlinks потрапляють шумові файли (`node_modules`, шаблони промптів, чернетки ADR, специфікації). `foam.files.ignore` тримає індекс сфокусованим на `docs/adr/` і `docs/ci4/`.

- [ ] **Step 1: Прочитати поточний `.vscode/settings.json`**

Run: відкрити `.vscode/settings.json` через Read.
Expected: JSON-обʼєкт. Останній ключ — `"rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"]`, далі закривна `}`.

- [ ] **Step 2: Додати ключ `foam.files.ignore` останнім**

Замінити рядок:

```json
  "rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"]
}
```

на:

```json
  "rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"],
  "foam.files.ignore": [
    "**/node_modules/**",
    ".claude/**",
    ".cursor/**",
    "docs/superpowers/**",
    "docs/ci4/_templates/**",
    "docs/adr/_inbox/**"
  ]
}
```

Зверни увагу: після `["app/src-tauri/Cargo.toml"]` додається кома. Решта ключів файлу не змінюється.

- [ ] **Step 3: Перевірити, що JSON валідний**

Run: `bun -e "JSON.parse(await Bun.file('.vscode/settings.json').text()); console.log('valid')"`
Expected: `valid`.

- [ ] **Step 4: Перевірити, що ключ присутній**

Run: `grep -A 8 'foam.files.ignore' .vscode/settings.json`
Expected: блок з 6 ignore-патернами.

- [ ] **Step 5: `git status && git diff .vscode/settings.json` і зупинка**

Run: `git status && git diff .vscode/settings.json`
Очікувано: додано ключ `foam.files.ignore` з 6 патернами; кома після рядка `rust-analyzer.linkedProjects`. Зупинись для перегляду.

---

## Task 3: Додати секцію-інструкцію в `docs/ci4/README.md`

**Files:**

- Modify: `docs/ci4/README.md`

**Why:** Інженер має знати, що рекомендовані розширення увімкнули, як відкрити preview з Mermaid, як користуватися backlinks. `docs/ci4/README.md` — рукотворний хаб C4-моделі, docs:regen його не регенерує.

- [ ] **Step 1: Прочитати поточний `docs/ci4/README.md`**

Run: відкрити `docs/ci4/README.md` через Read.
Expected: файл із секціями `## Що таке MLMaiL`, `## Рівні C4-моделі`, `## Статус проєкту MLMaiL`, `## Як читати документацію MLMaiL` (нумерований список з 3 пунктів), `## Перевірка` (останньою).

- [ ] **Step 2: Вставити нову секцію перед `## Перевірка`**

Знайти рядок:

```markdown
## Перевірка
```

і вставити **перед ним** новий блок (з порожнім рядком після нього, щоб `## Перевірка` лишилась відокремленою):

```markdown
## Читання документації MLMaiL у VS Code

Встанови рекомендовані розширення — VS Code запропонує їх при відкритті
репозиторію MLMaiL (`foam.foam-vscode` і `bierner.markdown-mermaid`). Тоді:

- **Preview** (`Cmd+K V`) — Mermaid-діаграми у файлах C4-моделі MLMaiL
  рендеряться автоматично, без сирого коду.
- **Cmd+Click** по лінку — перехід між ADR і C4-файлами MLMaiL.
- **Панель Connections** (Foam) — backlinks: які ADR сформували поточний
  C4-файл MLMaiL.
- **Foam: Show Graph** — загальна карта зв'язків ADR↔C4 моделі MLMaiL.
```

- [ ] **Step 3: Перевірити структуру файлу**

Run: `grep -n '^## ' docs/ci4/README.md`
Expected: секції в порядку — `Що таке MLMaiL`, `Рівні C4-моделі`, `Статус проєкту MLMaiL`, `Як читати документацію MLMaiL`, `Читання документації MLMaiL у VS Code`, `Перевірка`.

- [ ] **Step 4: Lint регенерованих/рукотворних markdown-файлів docs/ci4**

Run: `bunx markdownlint-cli2 docs/ci4/README.md`
Expected: `Summary: 0 error(s)`.

Якщо є помилки — виправити в `docs/ci4/README.md` (наприклад, порожні рядки навколо списку, довжина рядків), перезапустити.

- [ ] **Step 5: `git status && git diff docs/ci4/README.md` і зупинка**

Run: `git status && git diff docs/ci4/README.md`
Очікувано: додано одну секцію `## Читання документації MLMaiL у VS Code` перед `## Перевірка`. Зупинись для перегляду.

---

## Task 4: Ручна перевірка в VS Code

**Files:** жодних змін — лише валідація.

**Why:** Foam і Mermaid — конфігурація редактора; автотестів немає. Перевірка виконується вручну у VS Code за кроками зі spec.

- [ ] **Step 1: Відкрити репозиторій і встановити розширення**

Відкрити корінь mlmail у VS Code. Прийняти підказку «This workspace has extension recommendations» → встановити `foam.foam-vscode` і `bierner.markdown-mermaid`. Перезапустити VS Code, якщо вимагається.

- [ ] **Step 2: Перевірити рендер Mermaid**

Відкрити `docs/ci4/01-context.md` → `Cmd+K V` (Markdown Preview).
Expected: Mermaid-діаграма рендериться як графіка, не як сирий ` ```mermaid ` блок.

- [ ] **Step 3: Перевірити навігацію по лінках**

У `docs/ci4/01-context.md` (або будь-якому ADR із sentinel-блоком `**Опрацьовано**`) зробити `Cmd+Click` по markdown-лінку на C4-файл.
Expected: VS Code відкриває цільовий файл.

- [ ] **Step 4: Перевірити backlinks**

Відкрити `docs/ci4/03-components.md`. Відкрити панель Connections — вона у контейнері Foam в боковій панелі Explorer (за потреби — через Command Palette, пошук «Connections»).
Expected: показано backlinks — ADR-файли, що посилаються на `03-components.md` через sentinel-мітки.

- [ ] **Step 5: Перевірити граф і чистоту індексу**

Command Palette → `Foam: Show Graph`.
Expected: видно 30+ вузлів; ребра з'єднують вузли ADR і C4. Вузлів із `docs/ci4/_templates/`, `.claude/`, `docs/superpowers/` у графі немає.

- [ ] **Step 6: Фінальний `git status && git diff` і зупинка**

Run: `git status && git diff`
Очікувано: модифіковані `.vscode/extensions.json`, `.vscode/settings.json`, `docs/ci4/README.md`. Зупинись — користувач переглядає і вирішує щодо коміту.

---

## Підсумок

Після Task 4:

- `.vscode/extensions.json` — `foam.foam-vscode` + `bierner.markdown-mermaid` у рекомендаціях.
- `.vscode/settings.json` — ключ `foam.files.ignore` з 6 патернами.
- `docs/ci4/README.md` — секція «Читання документації MLMaiL у VS Code».
- Жодних змін у docs:regen, `package.json`, git-хуках, ADR-форматі.

Інженер, що відкриває mlmail у VS Code, отримує: відрендерені Mermaid-діаграми у preview, `Cmd+Click`-навігацію між ADR і C4, панель backlinks, і — як бонус — граф зв'язків.
