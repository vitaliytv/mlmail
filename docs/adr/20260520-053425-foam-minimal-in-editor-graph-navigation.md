---
session: 74cd5ade-0c55-4247-9bcf-493d2f6787ec
captured: 2026-05-20T05:34:25+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/74cd5ade-0c55-4247-9bcf-493d2f6787ec.jsonl
---

## ADR Foam Minimal — In-Editor Graph Navigation

## Context and Problem Statement

У mlmail існує граф зв'язків ADR↔C4-проекції (дані є в `manifest.json`, sentinel-мітках та `decisions.md`), але інженери не мають зручного in-editor способу навігувати ними — відкривають файли вручну. Потрібен засіб для навігації по графу безпосередньо у VS Code.

## Considered Options

* **A — Foam Minimal:** додати `foam.foam-vscode` у рекомендації, виставити `foam.files.ignore` для фокусування графа на `docs/adr/**` + `docs/ci4/**`.
* **B — Foam + збагачені forward-лінки:** додатково підкрутити шаблони docs:regen, щоб проекції емітили явні `[slug](../adr/slug.md)`, тригерить повний LLM-regen.
* **C — Foam + рукотворний hub-вузол:** окремий центральний граф-файл. `docs/ci4/README.md` вже є хабом.
* **Mermaid-граф з manifest.json:** генерувати статичну діаграму в `decisions.md` — не інтерактивна, не навігована.

## Decision Outcome

Chosen option: "A — Foam Minimal", because зміна тільки трьох файлів (`.vscode/extensions.json`, `.vscode/settings.json`, `docs/ci4/README.md`) без жодних змін у docs:regen, форматі ADR або npm-залежностях; Foam читає наявні markdown-лінки та автоматично будує backlinks у панелі Connections.

### Consequences

* Good, because transcript фіксує очікувану користь: 32+ вузли графа і backlinks для кожного файлу без зміни pipeline.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файли: `.vscode/extensions.json`, `.vscode/settings.json` (ключ `foam.files.ignore`), `docs/ci4/README.md`. Spec: `docs/superpowers/specs/2026-05-20-foam-graph-design.md`. Foam підтримує стандартні markdown-лінки `[text](path.md)` як ребра графа поряд з `[[wikilinks]]` — wikilinks у проєкт не вводяться, щоб зберегти рендер на GitHub і в mdbook.

---

## ADR docs-regen — LLM-Based C4 Documentation Regeneration Pipeline

## Context and Problem Statement

C4-документація (`docs/ci4/*.md`) дублювала інформацію з ADR і швидко застарівала. Потрібен автоматизований спосіб підтримувати її актуальною на основі clean ADR у `docs/adr/`.

## Considered Options

Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "LLM-based regeneration через `claude` CLI з модульним ESM-скриптом", because user визначив підхід на старті: читати всі clean ADR, передавати їх у промпт до LLM разом з шаблоном проекції, записувати результат у `docs/ci4/`. Вибір `claude` CLI (не Anthropic SDK прямо) зумовлений наявністю аутентифікованого CLI на машині розробника.

Pipeline (`scripts/docs-regen.js` + 11 ESM-модулів у `scripts/docs-regen/`):
- `discover.js` — знаходить clean ADR через globby + gray-matter.
- `manifest.js` — tracking-стан (`docs/ci4/manifest.json`): хеші правил, шаблонів, час обробки кожного ADR.
- `triggers.js` — визначає, що потребує регенерації (unmarked ADR, зміни rules/templates).
- `marks.js` — читає/записує sentinel-блок `**Опрацьовано** YYYY-MM-DD. Проекції: …` в кінець ADR.
- `projection.js` — збирає промпт, викликає LLM (з retry ≤ 3), парсить JSON-відповідь, зберігає debug-дамп при помилці.
- `lock.js` — `O_EXCL`-файловий lock (`docs/ci4/.docs-regen.lock`) для запобігання паралельних запусків.
- Флаги: `--dry`, `--check` (exit 1 при drift), `--projection <name>`, `--no-mark`.

### Consequences

* Good, because transcript фіксує підтверджену роботу: 51 юніт-тест pass, 0 fail; smoke-run — 5 проекцій згенеровано, 26 ADR помічено, idempotence перевірена (`OK, nothing to regenerate`).
* Bad, because transcript фіксує: LLM може повернути відповідь без поля `content` (перший smoke упав на `03-components`); вирішено retry + debug-дамп у `docs/ci4/.regen-debug/`. Також `markdownlint MD060` (padded таблиці) потребував post-processing `perl`-скриптом і додаткової інструкції у `_global.prompt.md`.

## More Information

Ключові файли: `scripts/docs-regen.js`, `scripts/docs-regen/{cli,discover,hash,lock,log,llm,manifest,marks,projection,templates,triggers}.js`, `scripts/docs-regen/default-templates/*.prompt.md`, `docs/ci4/manifest.json`, `.cursor/skills/docs-regen/SKILL.md`. Команда: `bun run docs:regen`. Тести: `scripts/__tests__/docs-regen/` (8 файлів, 51 тест). Worktree-safe детекція merge/rebase: `git rev-parse --git-dir` замість `stat('.git/MERGE_HEAD')` — виправлено після ENOTDIR у worktree.
