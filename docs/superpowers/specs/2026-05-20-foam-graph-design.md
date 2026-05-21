# In-editor навігація і перегляд документації через Foam — design spec

**Дата:** 2026-05-20
**Статус:** Approved, готовий до плану імплементації
**Scope:** репозиторій mlmail.
**Мета:** інженер навігує і читає C4/ADR-документацію всередині VS Code — з відрендереними Mermaid-діаграмами, переходом `Cmd+Click` по лінках і backlinks — без змін у пайплайні docs:regen чи у форматі markdown.

## Проблема

Інженер, що працює в `docs/adr/` або `docs/ci4/`, має три точки тертя:

1. **Mermaid-діаграми не рендеряться** у стандартному Markdown Preview VS Code — замість діаграми видно сирий code-блок. `docs/ci4/01-context.md` містить Mermaid-діаграму, згенеровану docs:regen.
2. **Немає backlinks** — у редакторі немає способу побачити, які ADR сформували конкретний C4-файл. Дані існують у sentinel-мітках `**Опрацьовано**` і в `decisions.md`, але вимагають ручного обходу кількох файлів.
3. **Навігація однобічна** — `Cmd+Click` по markdown-лінку працює нативно у VS Code, але зворотного напрямку (backlink) немає.

## Обраний підхід — Foam + Mermaid

Додати два VS Code-розширення як workspace-рекомендації і налаштувати ignore-список Foam. Без wikilinks, без змін у docs:regen, без npm-залежностей.

**Відкинуті альтернативи:**

- Генерувати Mermaid-граф з `manifest.json` — корисно як вторинний вигляд, але не інтерактивно і не навіговно.
- Додати `[[wikilinks]]` — ламає рендер на GitHub і в mdbook.
- Тільки Foam, без `bierner.markdown-mermaid` — Mermaid-діаграми у C4-файлах лишаються невідрендереними, що прямо б'є по «перегляді».

## Архітектура

### Наявна структура графа (змін не потребує)

Foam пасивно індексує весь markdown у воркспейсі й будує граф з наявних лінків:

| Тип ребра       | Джерело                                       | Ціль                                      |
| --------------- | --------------------------------------------- | ----------------------------------------- |
| ADR → проекції  | sentinel-блок `**Опрацьовано**` у кожному ADR | `docs/ci4/01-context.md` тощо             |
| decisions → ADR | `docs/ci4/decisions.md`                       | `docs/adr/*.md`                           |
| README-хаб → C4 | `docs/ci4/README.md`                          | `docs/ci4/01-context.md` … `decisions.md` |

26 ADR-вузлів + 5 C4-вузлів + 1 README-хаб ≈ 32 вузли. Ребра ADR→проекція беруться з sentinel-міток (кожен ADR посилається на 1-5 проекцій).

### Зміни

**`.vscode/extensions.json`**

Додати у масив `recommendations`:

- `"foam.foam-vscode"` — backlinks (панель Connections), автодоповнення лінків, sync лінків при rename.
- `"bierner.markdown-mermaid"` — рендер Mermaid-діаграм у вбудованому Markdown Preview VS Code.

VS Code пропонує інженеру встановити обидва при відкритті репозиторію. Наявні `recommendations` і `unwantedRecommendations` не чіпаються.

**`.vscode/settings.json`**

Додати один ключ до наявного файлу налаштувань:

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

Це тримає індекс Foam сфокусованим на вузлах `docs/adr/` і `docs/ci4/`. Кореневі markdown-файли (`AGENTS.md`, `CLAUDE.md`, `README.md`, `CHANGELOG.md`) лишаються периферійними вузлами — прийнятний шум. Решта налаштувань `.vscode/settings.json` не змінюється.

**`docs/ci4/README.md`**

Додати коротку секцію «Читання документації MLMaiL у VS Code»:

```markdown
## Читання документації MLMaiL у VS Code

Встанови рекомендовані розширення (VS Code запропонує при відкритті репозиторію MLMaiL). Тоді:

- **Preview** (`Cmd+K V`) — Mermaid-діаграми рендеряться автоматично.
- **Cmd+Click** по лінку — перехід між ADR і C4-файлами MLMaiL.
- **Панель Connections** (Foam) — backlinks: які ADR сформували поточний файл.
- **Foam: Show Graph** — загальна карта зв'язків ADR↔C4 MLMaiL.
```

`docs/ci4/README.md` — рукотворний файл, docs:regen його не регенерує, тож редагувати безпечно.

## Що НЕ змінюється

- Пайплайн docs:regen, його шаблони, формат виводу.
- Формат markdown в ADR (жодних wikilinks).
- `package.json`, git-хуки, CI.
- npm-залежності — Foam і `bierner.markdown-mermaid` це VS Code-розширення, не пакети.

## Перевірка

Тільки ручна (це конфігурація редактора, автотестів немає):

1. Відкрити VS Code у корені mlmail → прийняти підказки встановлення розширень (Foam + Mermaid).
2. Відкрити `docs/ci4/01-context.md` → `Cmd+K V` → Mermaid-діаграма рендериться (не сирий код).
3. `Cmd+Click` по лінку в sentinel-блоці ADR → перехід на C4-проекцію.
4. Відкрити `docs/ci4/03-components.md` → панель Connections показує backlinks від ADR-файлів.
5. `Foam: Show Graph` → видно 30+ вузлів, ребра з'єднують вузли ADR↔C4.
6. Вузли `docs/ci4/_templates/` і `.claude/` відсутні в графі.
