# docs-regen: LLM-Based Pipeline для Регенерації C4-Документації

**Status:** Accepted
**Date:** 2026-05-20

## Context and Problem Statement

C4-документація (`docs/ci4/*.md`) дублювала інформацію з ADR і швидко застарівала. Потрібен автоматизований спосіб підтримувати її актуальною на основі clean ADR у `docs/adr/`.

## Considered Options

- LLM-based regeneration через `claude` CLI з модульним ESM-скриптом
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "LLM-based regeneration через `claude` CLI з модульним ESM-скриптом", because user визначив підхід на старті: читати всі clean ADR, передавати їх у промпт до LLM разом з шаблоном проекції, записувати результат у `docs/ci4/`. Вибір `claude` CLI (не Anthropic SDK прямо) зумовлений наявністю аутентифікованого CLI на машині розробника.

Pipeline (`scripts/docs-regen.js` + 11 ESM-модулів у `scripts/docs-regen/`):

- `discover.js` — знаходить clean ADR через tinyglobby + gray-matter.
- `manifest.js` — tracking-стан (`docs/ci4/manifest.json`): хеші правил, шаблонів, час обробки кожного ADR.
- `triggers.js` — визначає, що потребує регенерації (unmarked ADR, зміни rules/templates).
- `marks.js` — читає/записує sentinel-блок `**Опрацьовано** YYYY-MM-DD. Проекції: …` в кінець ADR.
- `projection.js` — збирає промпт, викликає LLM (з retry ≤ 3), парсить JSON-відповідь, зберігає debug-дамп при помилці.
- `lock.js` — `O_EXCL`-файловий lock (`docs/ci4/.docs-regen.lock`) для запобігання паралельних запусків.
- Флаги: `--dry`, `--check` (exit 1 при drift), `--projection <name>`, `--no-mark`.

### Consequences

- Good, because transcript фіксує підтверджену роботу: 51 юніт-тест pass, 0 fail; smoke-run — 5 проекцій згенеровано, 26 ADR помічено, idempotence перевірена (`OK, nothing to regenerate`).
- Bad, because LLM може повернути відповідь без поля `content` (перший smoke упав на `03-components`); вирішено retry + debug-дамп у `docs/ci4/.regen-debug/`. Також `markdownlint MD060` (padded таблиці) потребував post-processing `perl`-скриптом і додаткової інструкції у `_global.prompt.md`.

## More Information

Ключові файли: `scripts/docs-regen.js`, `scripts/docs-regen/{cli,discover,hash,lock,log,llm,manifest,marks,projection,templates,triggers}.js`, `scripts/docs-regen/default-templates/*.prompt.md`, `docs/ci4/manifest.json`, `.cursor/skills/docs-regen/SKILL.md`. Команда: `bun run docs:regen`. Тести: `scripts/__tests__/docs-regen/` (8 файлів, 51 тест). Worktree-safe детекція merge/rebase: `git rev-parse --git-dir` замість `stat('.git/MERGE_HEAD')` — виправлено після ENOTDIR у worktree.

Також у цій сесії: Foam Minimal (`foam.foam-vscode`) додано у `.vscode/extensions.json` і `foam.files.ignore` у `.vscode/settings.json` для in-editor graph navigation ADR↔C4 — окреме рішення, не зафіксоване окремим ADR у цьому батчі.
