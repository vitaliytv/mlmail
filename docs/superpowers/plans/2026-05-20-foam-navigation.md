# Foam In-Editor Navigation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable engineers to navigate and read C4/ADR documentation in VS Code with rendered Mermaid diagrams and backlinks via two recommended extensions.

**Architecture:** Three config-only changes — add two VS Code extensions to workspace recommendations, add Foam's ignore list to suppress noise nodes, extend `docs/ci4/README.md` with a navigation guide. No code, no npm deps.

**Tech Stack:** VS Code workspace config (JSON), Markdown.

**Spec:** `docs/superpowers/specs/2026-05-20-foam-graph-design.md`

---

### Task 1: Add extensions to `.vscode/extensions.json`

**Files:**

- Modify: `.vscode/extensions.json`

Current `recommendations` array has 11 extensions. Add two at the end.

- [ ] **Step 1: Read the file**

```bash
cat .vscode/extensions.json
```

- [ ] **Step 2: Add the two extensions**

Open `.vscode/extensions.json`. Locate `"recommendations"` array. Append two entries:

```json
"foam.foam-vscode",
"bierner.markdown-mermaid"
```

Result (tail of recommendations array):

```json
    "tauri-apps.tauri-vscode",
    "rust-lang.rust-analyzer",
    "foam.foam-vscode",
    "bierner.markdown-mermaid"
```

- [ ] **Step 3: Verify JSON is valid**

```bash
python3 -m json.tool .vscode/extensions.json > /dev/null && echo "OK"
```

Expected: `OK`

- [ ] **Step 4: Review changes**

```bash
git status && git diff .vscode/extensions.json
```

---

### Task 2: Add `foam.files.ignore` to `.vscode/settings.json`

**Files:**

- Modify: `.vscode/settings.json`

Adds one top-level key to the existing settings object to prevent noisy files from appearing in the Foam graph.

- [ ] **Step 1: Read the file**

```bash
cat .vscode/settings.json
```

- [ ] **Step 2: Add `foam.files.ignore`**

Inside the root JSON object (anywhere, e.g. before the closing `}`), add:

```json
"foam.files.ignore": [
  "**/node_modules/**",
  ".claude/**",
  ".cursor/**",
  "docs/superpowers/**",
  "docs/ci4/_templates/**",
  "docs/adr/_inbox/**"
]
```

- [ ] **Step 3: Verify JSON is valid**

```bash
python3 -m json.tool .vscode/settings.json > /dev/null && echo "OK"
```

Expected: `OK`

- [ ] **Step 4: Review changes**

```bash
git status && git diff .vscode/settings.json
```

---

### Task 3: Add navigation guide to `docs/ci4/README.md`

**Files:**

- Modify: `docs/ci4/README.md`

Add a short Ukrainian-language section. This file is hand-authored and not touched by docs:regen.

- [ ] **Step 1: Read current file**

```bash
cat docs/ci4/README.md
```

- [ ] **Step 2: Append navigation section**

Append to the end of `docs/ci4/README.md`:

```markdown
## Читання документації у VS Code

Встанови рекомендовані розширення (VS Code запропонує при відкритті репо). Тоді:

- **Preview** (`Cmd+K V`) — Mermaid-діаграми рендеряться автоматично
- **Cmd+Click** по лінку — перехід між ADR і C4-файлами
- **Connections panel** (Foam) — backlinks: які ADR сформували поточний файл
- **Foam: Show Graph** — загальна карта зв'язків ADR↔C4
```

- [ ] **Step 3: Verify markdownlint**

```bash
bunx markdownlint-cli2 docs/ci4/README.md 2>&1
```

Expected: `Summary: 0 error(s) found`

- [ ] **Step 4: Review changes**

```bash
git status && git diff docs/ci4/README.md
```

---

### Task 4: Manual smoke verification

**Files:** none modified

Verify the three changes work end-to-end in VS Code.

- [ ] **Step 1: Open VS Code at repo root**

Open VS Code in `mlmail/`. If prompted to install recommended extensions — accept. Install `foam.foam-vscode` and `bierner.markdown-mermaid`.

- [ ] **Step 2: Test Mermaid rendering**

Open `docs/ci4/01-context.md`. Press `Cmd+K V` (Markdown Preview). Verify the Mermaid diagram renders as a diagram (not a code block).

- [ ] **Step 3: Test navigation**

Open any ADR with a sentinel block (e.g. `docs/adr/ADR-0006-google-oauth.md`). `Cmd+Click` on `[01-context](../ci4/01-context.md)` → verifies it opens the file.

- [ ] **Step 4: Test backlinks**

Open `docs/ci4/03-components.md`. Open Foam Connections panel (sidebar). Verify backlinks from multiple ADR files appear.

- [ ] **Step 5: Test graph ignore**

`Foam: Show Graph`. Verify `docs/ci4/_templates/` and `.claude/` directories do NOT appear as graph nodes.
