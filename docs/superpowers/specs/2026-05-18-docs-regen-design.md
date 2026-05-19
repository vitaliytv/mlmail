# Регенератор документації MLMaiL з ADR — design spec

**Дата:** 2026-05-18
**Статус:** Approved, готовий до плану імплементації
**Scope:** Phase 1 — детермінований регенератор C4-документації MLMaiL у `docs/ci4/` з clean ADR у `docs/adr/` через LLM. Окрема команда `bun run docs:regen`, manifest `docs/ci4/manifest.json`, мітка «Опрацьовано» в ADR.

## Мета

1. Розробник MLMaiL пише clean ADR у `docs/adr/<slug>.md` (через `n-adr` normalize-flow або руками), запускає `bun run docs:regen` — і 5 C4-файлів у `docs/ci4/` оновлюються від цих ADR через LLM.
2. Кожен ADR після опрацювання отримує sentinel-блок «Опрацьовано», що перелічує проекції, у які LLM його включив (або «жодної»).
3. Manifest `docs/ci4/manifest.json` відстежує, які ADR опрацьовані (через мітку в самих ADR як джерело правди), хеш кожної проекції, хеші шаблонів і `n-ci4.mdc` — на наступному запуску регенерується лише те, що змінилось.
4. Регенератор працює тільки явною командою — без Stop-hook, без авто-тригерів, без push-ів.

Що **поза scope** цієї ітерації (Phase 1):

- Author-flow `/docs-propose` (інтерактивне знаходження релевантних ADR, draft нового ADR з варіантами, impact-аналіз) — Phase 2, окрема спека.
- Stop-hook авто-регенерації.
- CI-перевірка drift на pull-request (CLI-режим `--check` реалізується, але як saute, не як GitHub Action).
- Багатомовні проекції — українська за дефолтом.
- Кеш LLM-відповідей між запусками.
- Зміна `n-adr` capture/normalize-flow — він живе окремо, як зараз.
- Регенерація `docs/ci4/README.md` — лишається рукотворним вступом.
- Інкорпорація drafts (файли з `session:` у frontmatter) — їх ігноруємо, вони чекають `n-adr` normalize-flow.

## Контекст і обмеження

### Інваріанти з `.cursor/rules/n-ci4.mdc`

Правило `n-ci4.mdc` диктує канон, який регенератор зобов'язаний дотримуватись у згенерованому виводі:

- **Markdown — джерело істини**, без HTML/CSS-розмітки. Жодних `<div>`, `<span>`, класів, навігаційних обгорток у регенерованих файлах.
- **Контекстна незалежність розділів.** Кожен заголовок і абзац у регенерованому файлі повинен мати сенс **без сусіднього тексту**. Заборона на «як було згадано вище», «цей сервіс», «той самий компонент». Замість цього — щоразу повторювати назву сутності: «контейнер `tauri-backend`», «функція `summarize_email()`», «зовнішня система `Gmail API`».
- **Spec-as-Source.** Регенерована документація MLMaiL — первинний артефакт, з якого LLM-агент відновлює код. ADR описує **намір**, проекція описує **систему, як вона має бути**.
- **Rebuild Test.** Регенерована проекція вважається повною, якщо LLM з нею (плюс ADR-и) може відтворити відповідну ділянку коду MLMaiL. Прогалини — баг проекції, не «складність задачі».
- **`docs/ci4/` — канонічне місце** для C4-схем. Регенератор пише тільки туди.
- **Трасування для недетермінованих компонентів.** Для компонентів MLMaiL з LLM- або евристичною поведінкою (summary-engine, tts-engine у майбутньому) — обов'язкова секція з посиланням на дашборд/storage трасувань. Поки інфраструктури немає — плейсхолдер `TBD: tracing-storage`.
- **Зв'язок із тестами.** Кожен компонент MLMaiL у регенерованій проекції — з посиланням на тести (`app/src-tauri/tests/<name>.rs`, `app/src/__tests__/<name>.test.js`). Якщо тестів ще немає — `TBD: tests`.

### Поточний стан репозиторію MLMaiL

Станом на 2026-05-18:

- У `docs/adr/` — 26 clean ADR (без `session:` у frontmatter). `_inbox/` відсутній.
- У `docs/ci4/` — 5 рукотворних файлів: `01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`, плюс `README.md`. Це **цільова архітектура** MLMaiL — описує систему «куди агент дотягує код», не лише поточний стартовий каркас.
- Існує `docs/superpowers/specs/` з попередніми design spec'ами — ця спека стає черговою.
- `package.json` — workspace з одним пакетом `app/`, `type: module`, `engines: { bun: ">=1.3", node: ">=24" }`.

Кожен з 26 clean ADR — потенційний вхід регенератора Phase 1.

## Архітектура високого рівня

```
┌──────────────────────────────────────────────────────────────────────┐
│ docs/adr/<slug>.md   (clean ADR, 26 шт, читання)                    │
│   ↓                                                                  │
│   ↓  glob + filter (no session: frontmatter)                         │
│   ↓                                                                  │
├─→ scripts/docs-regen.js                                              │
│     ├─ Прочитати manifest docs/ci4/manifest.json                     │
│     ├─ Знайти ADR без мітки + diff шаблонів і правил                 │
│     ├─ Якщо є тригер — регенерувати всі 5 проекцій:                  │
│     │    ├─ Зібрати inputs (template + global + всі ADR body +       │
│     │    │   поточний файл проекції)                                 │
│     │    ├─ Викликати LLM (claude → cursor-agent fallback)           │
│     │    └─ Прочитати JSON-відповідь { content, used_adrs }          │
│     ├─ Записати docs/ci4/<projection>.md                             │
│     ├─ Оновити sentinel-блок у кожному clean ADR                     │
│     └─ Записати manifest                                             │
│   ↓                                                                  │
│   ↓                                                                  │
├─→ docs/ci4/01-context.md, 02-containers.md, 03-components.md,        │
│   04-code.md, decisions.md  (5 регенерованих проекцій)               │
│                                                                      │
└─→ docs/ci4/manifest.json  (хеші + tracking)                          │
```

Жодного автоматичного тригера. Розробник запускає `bun run docs:regen` свідомо, переглядає `git status && git diff`, комітить як `docs:` ком.

## ADR як вхід регенератора

### Дискавері clean ADR

Регенератор робить `glob('docs/adr/**/*.md')` з виключенням:

- `docs/adr/_inbox/**` — drafts, які чекають `n-adr` normalize-flow.
- Файли з YAML frontmatter, що містить поле `session:` — drafts capture-decisions.

Для решти — ADR вважається **clean** і йде в inputs. Парсинг frontmatter через `gray-matter`. Слаг ADR — ім'я файлу без `.md` (наприклад `quasar-2-ui-фреймворк-mlmail`, `ADR-0006-google-oauth`).

### Обмеження clean ADR

Регенератор **не змінює body** clean ADR, не нормалізує, не перейменовує, не править frontmatter. Єдина модифікація — sentinel-блок «Опрацьовано» в кінці файлу (формат — у секції «Мітка `Опрацьовано` в ADR»).

### Колізії

Слаг ADR має бути унікальним у межах `docs/adr/**`. Якщо два ADR-файли мають однакове ім'я (різні підкаталоги) — регенератор виходить з помилкою `ADR slug collision: <slug>`.

## Проекції як вихід регенератора

Регенератор пише **рівно 5 файлів** у `docs/ci4/`:

| Файл | Аудиторія | Призначення |
| --- | --- | --- |
| `docs/ci4/01-context.md` | менеджер MLMaiL + інженер MLMaiL | System Context (C4 рівень 1): користувач MLMaiL, зовнішні системи (Google Identity, Gmail API, LLM-провайдер, TTS-провайдер), use-cases (фічі високого рівня), поточний стан |
| `docs/ci4/02-containers.md` | інженер MLMaiL (менеджер для огляду) | Containers (C4 рівень 2): Vue 3 frontend, Tauri 2 + Rust backend, локальне сховище нотаток `home/`/`work/`, відповідальності, дані, інтерфейси |
| `docs/ci4/03-components.md` | інженер MLMaiL | Components (C4 рівень 3): логічні компоненти всередині кожного контейнера MLMaiL — `gmail-client`, `oauth-store`, `summary-engine`, `tts-engine`, `notes-store`, `action-handler`, Vue views та stores |
| `docs/ci4/04-code.md` | інженер MLMaiL | Code (C4 рівень 4): Tauri-команди (сигнатури + файл), Vue-компоненти ключових екранів (props + файл), конфігурація (`tauri.conf.json`, env vars), operations (збірка, запуск) |
| `docs/ci4/decisions.md` | обидва | Зведення ADR-впливів на C4: хронологічний індекс ADR, для кожного — на які рівні C4 вплинув, зворотний індекс (для кожного рівня C4 — які ADR його сформували), superseded chains |

Файл `docs/ci4/README.md` — **рукотворний вступ**, регенератором не чіпається. Якщо людина хоче — оновлює руками.

### Аудиторії і вибір ADR

Регенератор не фільтрує ADR за tags/components — повний набір clean ADR іде в кожен LLM-запит. **LLM сам вирішує**, які з ADR релевантні для якої проекції, на основі prompt-шаблону, що описує призначення проекції. LLM повертає список використаних ADR у полі `used_adrs` своєї JSON-відповіді — це і є джерело для мітки «Опрацьовано» в ADR.

### Зміст C4-проекцій MLMaiL

Регенератор інкорпорує тематичні аспекти (features, data-model, operations, security) **всередину** відповідних C4-рівнів, не виносячи їх в окремі файли:

- **Features** живуть як use-cases у `docs/ci4/01-context.md` (auth через Google, fetch листів з Gmail, AI-саммері листа, озвучення саммері, дії над листом: видалити / видалити+фільтр / `home` нотатка / `work` нотатка).
- **Data-model** — як власність контейнера у `docs/ci4/02-containers.md` (наприклад, локальне сховище нотаток володіє `.md`-файлами у `home/` і `work/`; Tauri backend володіє OAuth-токенами у файловому сторі).
- **Operations** — секція у `docs/ci4/04-code.md`: збірка macOS app bundle, Android APK, env vars, deploy.
- **Security** — cross-cutting concern, який регенератор просить LLM згадати у кожному рівні, де релевантний (наприклад, OAuth scopes у `01-context.md`, межі довіри між контейнерами в `02-containers.md`, секрети в `04-code.md`).

Це відповідає правилу `n-ci4.mdc` «інших місць для C4-схем у репозиторії немає».

## Шаблони промптів

Регенератор зберігає шаблони у `docs/ci4/_templates/`. Шаблони — частина репо, редагуються людиною, **не регенеруються**.

```
docs/ci4/_templates/
├── _global.prompt.md          # глобальні правила оформлення (з n-ci4.mdc)
├── 01-context.prompt.md
├── 02-containers.prompt.md
├── 03-components.prompt.md
├── 04-code.prompt.md
└── decisions.prompt.md
```

### Глобальний фрагмент `_global.prompt.md`

Дослівні інваріанти з `.cursor/rules/n-ci4.mdc`, інжектуються у кожен LLM-запит:

- Чистий Markdown, нуль HTML/CSS/класів.
- Кожен заголовок і абзац самодостатній. Заборона на «вище», «згаданий», «той самий».
- Повторювати назви: «контейнер `tauri-backend`», «функція `summarize_email()`», не «він»/«його».
- Для недетермінованих компонентів MLMaiL — секція з трасуванням обов'язкова.
- Для компонентів MLMaiL — посилання на тести обов'язкові (або `TBD: tests`).
- Українська для UI/бізнес-частини, англійська для технічних ідентифікаторів.
- Якщо ADR описує `planned`-рішення — компонент іде в розділ `Поточний стан → Planned`, не в `Поточний стан → Реалізовано`.

### Per-projection шаблон

Шаблон визначає для своєї проекції:

- Аудиторію.
- Обов'язкові секції.
- Інваріанти структури.
- Приклади формулювань (1-2 коротких).

Структура шаблону:

```markdown
# Інструкції LLM для генерації <projection-name>.md

## Аудиторія
<хто читає>

## Призначення файлу
<що цей файл відповідає на>

## Обов'язкові секції
1. <секція>: <опис, що тут має бути>
...

## Інваріанти
- <інваріант>
- ...

## Формат виводу
JSON: { "content": "<повний markdown файлу>", "used_adrs": ["<slug>", ...] }

## Приклади формулювань
"Контейнер `tauri-backend` MLMaiL реалізує..."  не "Backend реалізує..."
```

### Зміст обов'язкових секцій по проекціях

#### `01-context.prompt.md` — System Context MLMaiL

Обов'язкові секції регенерованого `docs/ci4/01-context.md`:

1. **Призначення MLMaiL** — 1 абзац бізнес-мовою, що робить застосунок і для кого.
2. **Користувачі MLMaiL** — хто, які цілі.
3. **Зовнішні системи MLMaiL** — Google Identity, Gmail API, LLM-провайдер, TTS-провайдер. Для кожної: що робить, які межі довіри, які scopes/permissions.
4. **Use-cases MLMaiL** — auth, fetch листів з Gmail, AI-саммері листа, озвучення саммері, дії над листом.
5. **Cross-cutting (приватність, безпека)** — на високому рівні.
6. **Поточний стан MLMaiL** — що реалізовано, що `planned`.

#### `02-containers.prompt.md` — Containers MLMaiL

Обов'язкові секції регенерованого `docs/ci4/02-containers.md`:

1. **Список контейнерів MLMaiL.** Для кожного:
   - Технологія (повна назва).
   - Відповідальність.
   - Дані, якими володіє (data-model тут).
   - Інтерфейси з іншими контейнерами та зовнішніми системами (напрямки залежностей).
   - Розгортання (macOS app bundle, Android APK).
2. **Cross-cutting** — спільна конфігурація, секрети, межі довіри.

#### `03-components.prompt.md` — Components MLMaiL

Обов'язкові секції регенерованого `docs/ci4/03-components.md`:

1. **Компоненти кожного контейнера MLMaiL.** Для кожного компонента:
   - Відповідальність (1-2 речення, самодостатньо).
   - Залежності (входи/виходи).
   - Посилання на тести (`app/src-tauri/tests/<name>.rs`, `app/src/__tests__/<name>.test.js` або `TBD: tests`).
   - Для недетермінованих компонентів MLMaiL — посилання на трасування (`TBD: tracing-storage`).
   - Релевантні ADR (slug-и).

#### `04-code.prompt.md` — Code MLMaiL

Обов'язкові секції регенерованого `docs/ci4/04-code.md`:

1. **Tauri-команди MLMaiL** — сигнатури, файл (`app/src-tauri/src/...`).
2. **Vue-компоненти MLMaiL** ключових екранів — props, файл.
3. **Конфігурація MLMaiL** — `tauri.conf.json`, env vars, secrets, .env-файли.
4. **Operations MLMaiL** — збірка, запуск, deploy macOS app bundle / Android APK.

#### `decisions.prompt.md` — ADR індекс MLMaiL

Обов'язкові секції регенерованого `docs/ci4/decisions.md`:

1. **Хронологічний індекс ADR MLMaiL.** Для кожного — slug, дата, статус, однорядковий summary.
2. **Вплив ADR на рівні C4 MLMaiL.** Для кожного ADR — на які рівні (`01-context`, `02-containers`, `03-components`, `04-code`) вплинув. Поле для LLM-висновку.
3. **Зворотний індекс.** Для кожного рівня C4 MLMaiL — які ADR його сформували.
4. **Superseded chains** — `A → B → C`, якщо ADR взаємно заміщують одне одного.

## Мітка `Опрацьовано` в ADR

### Формат

Регенератор додає (або переписує) **останній блок** ADR-файлу за схемою:

```markdown
---

**Опрацьовано** 2026-05-18. Проекції:
- [01-context](../ci4/01-context.md)
- [03-components](../ci4/03-components.md)
- [decisions](../ci4/decisions.md)
```

Якщо ADR було передано в LLM, але LLM не включив його в жодну з 5 проекцій:

```markdown
---

**Опрацьовано** 2026-05-18. Проекції: жодної.
```

**Sentinel:** markdown horizontal rule `---` як окремий рядок перед блоком, плюс домовленість, що **останній** такий блок у файлі — мітка регенератора. Машинна форма (`["01-context", "03-components", "decisions"]` або `true` для «жодної») зберігається в `manifest.json`.

### Алгоритм оновлення мітки

Для кожного clean ADR:

1. Прочитати файл як рядок.
2. Знайти останнє входження `^---\s*$`, після якого йде абзац, що починається з `**Опрацьовано**`. Якщо знайдено — відрізати все від цього `---` до кінця файлу.
3. Сформувати новий блок мітки (з актуальним списком проекцій або «жодної»).
4. Дописати `\n\n---\n\n<блок мітки>\n` у кінець файлу.
5. Записати назад.

Дата мітки — поточна UTC у форматі `YYYY-MM-DD` (без часу).

### Заборона HTML

Sentinel **не** використовує HTML-коментар `<!-- ... -->`, бо правило `n-ci4.mdc` забороняє HTML-розмітку в `.md`. `---` — стандартний markdown horizontal rule, нейтральний для рендерерів.

## Manifest

### Шлях і життя

`docs/ci4/manifest.json` — закомічений у git, оновлюється кожним запуском `docs:regen` (атомарно: запис у `.manifest.json.tmp` + `rename`).

### Схема

```json
{
  "version": 1,
  "generated_at": "2026-05-18T16:30:00Z",
  "tool": {
    "name": "docs-regen",
    "version": "0.1.0",
    "model": "claude-sonnet-4.6"
  },
  "rules": {
    ".cursor/rules/n-ci4.mdc": { "hash": "sha256:..." }
  },
  "templates": {
    "_global.prompt.md":     { "hash": "sha256:..." },
    "01-context.prompt.md":  { "hash": "sha256:..." },
    "02-containers.prompt.md":{ "hash": "sha256:..." },
    "03-components.prompt.md":{ "hash": "sha256:..." },
    "04-code.prompt.md":     { "hash": "sha256:..." },
    "decisions.prompt.md":   { "hash": "sha256:..." }
  },
  "adrs": {
    "ADR-0006-google-oauth": {
      "path": "docs/adr/ADR-0006-google-oauth.md",
      "processed_at": "2026-05-18",
      "projections": ["01-context", "03-components", "decisions"]
    },
    "quasar-2-ui-фреймворк-mlmail": {
      "path": "docs/adr/quasar-2-ui-фреймворк-mlmail.md",
      "processed_at": "2026-05-18",
      "projections": []
    }
  },
  "projections": {
    "01-context": {
      "path": "docs/ci4/01-context.md",
      "output_hash": "sha256:...",
      "generated_at": "2026-05-18T16:30:00Z",
      "used_adrs": ["ADR-0006-google-oauth", "..."],
      "llm_input_tokens": 12345,
      "llm_output_tokens": 4567
    },
    "02-containers": { "...": "..." },
    "03-components": { "...": "..." },
    "04-code": { "...": "..." },
    "decisions": { "...": "..." }
  }
}
```

### Інваріанти manifest

- **Ключ ADR в `adrs`** — slug файла без `.md`, унікальний у межах `docs/adr/**`.
- **ADR не має поля `hash`.** Сигнал «опрацьовано» бере на себе мітка `**Опрацьовано**` в кінці тіла ADR. Регенератор перевіряє наявність мітки, не вміст ADR. Хочеш re-process ADR — видаляєш мітку з файлу руками (або використовуєш `--all`).
- **`projections` в ADR** і **`used_adrs` в projection** — взаємні. Дублювання навмисне: per-ADR view (де я опинився) і per-projection view (хто мене сформував). `processed_at` — дата останнього регену, в якому ADR брав участь.
- **`output_hash` проекції** зберігається, бо проекція — згенерований файл, і ми хочемо знати, чи людина не правила його руками між запусками регенератора (поза scope Phase 1, але дешево лишити).
- **`rules` і `templates` хеші** — їхня зміна форсить regen **усіх** 5 проекцій (правило диктує оформлення, шаблон — структуру).
- **Порядок ключів** — усі об'єкти сортуються лексикографічно перед `JSON.stringify(value, null, 2) + '\n'`. Це робить `git diff` мінімальним при кожному запуску.

### Що не зберігаємо в manifest

- Body ADR і хеш ADR (мітка в тілі ADR — джерело правди про «опрацьовано»).
- Body проекції (хеш — достатньо).
- LLM-відповіді та промпти (це у `.regen.log`, gitignored).

## Алгоритм регенератора

`scripts/docs-regen.js` запускається через `bun run docs:regen`. Послідовність:

### 1. Локи і санітарні перевірки

- `flock` на `.claude/hooks/.docs-regen.lock`. Якщо інший запуск тримає лок — вийти з повідомленням.
- Якщо репозиторій у `MERGE_HEAD` / `rebase-*` — миттєвий exit (правка дерева під час конфлікту небезпечна).

### 2. Дискавері clean ADR

- `glob('docs/adr/**/*.md')` з ignore `_inbox/**`.
- Для кожного — `gray-matter` парсить frontmatter. Якщо містить `session:` — пропустити.
- Решту — додати в список clean ADR з полями: `slug`, `path`, `body` (вміст без frontmatter і без блоку мітки), `has_mark` (boolean — чи є sentinel-блок `**Опрацьовано**` в кінці файлу).

### 3. Завантаження manifest

- Прочитати `docs/ci4/manifest.json`. Якщо відсутній — створити порожній (перший запуск).

### 4. Визначення, що треба регенерувати

Сигнали для тригера regen:

- `unmarked` — ADR без мітки `**Опрацьовано**` у body. Це або новий ADR (вперше), або такий, у якого людина свідомо видалила мітку, щоб re-process. Список slug-ів.
- `removed` — ADR є в `manifest.adrs`, але немає у файловій системі (видалений людиною).
- `rules_changed` — хеш `.cursor/rules/n-ci4.mdc` відрізняється від `manifest.rules`.
- `templates_changed` — хеш будь-якого `docs/ci4/_templates/*.prompt.md` відрізняється від `manifest.templates`.

Якщо `unmarked = []` і `removed = []` і `rules_changed = false` і `templates_changed = false` і CLI не передав `--all` — exit `OK, nothing to regenerate`.

Інакше — **усі 5 проекцій** позначаються «to regenerate» (бо вибірку ADR робить LLM, і неможливо передбачити, які саме він використає; будь-який сигнал — повний цикл).

Перед регенерацією регенератор друкує сигнали в stdout: `Triggers: 3 unmarked ADRs, 0 removed, rules changed: no, templates changed: no`.

### 5. Регенерація кожної проекції

Для кожної з 5 проекцій:

1. **Підготувати inputs bundle:**
   - Глобальний промпт-фрагмент `_global.prompt.md`.
   - Per-projection шаблон `<name>.prompt.md`.
   - Усі clean ADR (повним body, кожен обгорнутий маркером `### ADR: <slug>` + body).
   - Поточний вміст `docs/ci4/<name>.md` як «попередній стан» (для consistency).
   - Однорядкова інструкція: «Поверни рівно один JSON-обʼєкт `{ "content": "<markdown>", "used_adrs": ["<slug>", ...] }`, без markdown-обгортки, без пояснень».
2. **Викликати LLM:**
   - Перший вибір: `claude -p --model sonnet` (або модель з env `DOCS_REGEN_MODEL`).
   - Fallback: `cursor-agent -p --mode ask --output-format text --model claude-4.6-sonnet-medium`.
   - Жодного CLI у PATH — exit з ненульовим кодом, нічого не пишеться.
   - Передача через stdin.
3. **Розпарсити відповідь LLM:**
   - Знайти JSON-обʼєкт. Якщо тіло обгорнуте у ```` ```json … ``` ```` — відрізати fence.
   - Валідувати наявність полів `content` (рядок) і `used_adrs` (масив рядків).
   - Якщо парсинг провалився — retry 1 раз з підказкою «верни лише JSON, без markdown-fence». Другий провал → exit з ненульовим кодом, проекція **не** оновлюється, manifest **не** чіпається.
4. **Записати проекцію:**
   - `await Bun.write('docs/ci4/<name>.md', content)`.
   - Обчислити `sha256` нового вмісту, зберегти для manifest.

### 6. Оновлення міток в ADR

- Зібрати агрегований мапінг `adr_slug → projections_used` зі всіх `used_adrs` 5 проекцій.
- Перебрати **усі** clean ADR (не лише «changed»):
  - Якщо ADR з'явився у хоча б одній проекції — нова мітка з переліком посилань.
  - Якщо ADR прогнаний через LLM, але не з'явився ніде — мітка «Проекції: жодної.»
  - Алгоритм запису — у секції «Алгоритм оновлення мітки» вище.
- LLM повернув невідомий slug у `used_adrs` — warn у лог, у мітку не йде.

### 7. Запис manifest

- Скласти повний обʼєкт по схемі.
- Сортувати ключі лексикографічно.
- `await Bun.write('docs/ci4/manifest.json.tmp', json)` + `await rename(tmp, final)`.

### 8. Лог підсумку

У stdout і в `.regen.log` (gitignored, `docs/ci4/.regen.log`):

```
docs-regen 2026-05-18T16:30:00Z
- ADRs processed: 26
- Projections regenerated: 5 (01-context, 02-containers, 03-components, 04-code, decisions)
- Marks updated: 26 (24 with projections, 2 with "жодної")
- LLM calls: 5 (claude --model sonnet)
- Total tokens: input 61234, output 22890
```

## CLI поверхня

```
bun run docs:regen                       # повний цикл, тільки що треба
bun run docs:regen --projection 01-context  # тільки одна проекція
bun run docs:regen --all                 # форсити regen усіх 5, ігноруючи мітки та хеші шаблонів
bun run docs:regen --dry                 # планувати, не писати на диск і не викликати LLM
bun run docs:regen --no-mark             # не оновлювати мітки в ADR (debug)
bun run docs:regen --check               # CI-режим: fail якщо drift, без LLM-викликів
```

Прапор `--projection <name>` — регенерувати лише вказану проекцію. Імена: `01-context`, `02-containers`, `03-components`, `04-code`, `decisions`.

Прапор `--check` — детермінований, без LLM-викликів і без запису. Перевіряє наявність мітки в кожному clean ADR, а також хеші шаблонів і `n-ci4.mdc` проти manifest. Exit code:

- `0` — синхрон, нічого регенерувати.
- `1` — drift, треба запустити `bun run docs:regen`.
- `2` — невалідний стан (битий manifest, ADR slug collision, manifest відсутній).

`--check` входить в Phase 1 як CLI-можливість для **майбутнього CI gate** — підключення в `npx @nitra/cursor check` або GitHub Action — окрема задача поза цією спекою.

### Slash-команда `/docs-regen`

Створюється `.cursor/skills/docs-regen/SKILL.md` за зразком `.cursor/skills/n-adr-normalize/SKILL.md`. Тонкий wrapper:

- Опис: «Регенерація C4-документації MLMaiL з clean ADR».
- Тригери: «оновити документацію», «regen docs», «documentation», «c4 docs».
- Команда: `bun run docs:regen`.

SKILL.md не дублює логіку — лише дозволяє агенту викликати команду по запиту користувача.

## Інтеграція з n-adr і workflow

Жодних змін у `.cursor/rules/n-adr.mdc`. Регенератор живе поза capture/normalize-flow.

Типовий workflow розробника MLMaiL:

1. Робота в Claude/Cursor → Stop-hook `capture-decisions.sh` пише draft у `docs/adr/<timestamp>-<sid>.md`.
2. Періодично (поріг `ADR_NORMALIZE_THRESHOLD`) або руками через `/n-adr-normalize` → `normalize-decisions.sh` перетворює drafts на clean ADR (зняття frontmatter, kebab-slug, додавання `**Status: Accepted**` + `**Date:**`).
3. Розробник запускає `bun run docs:regen` → 5 файлів у `docs/ci4/` оновлюються, мітки в ADR оновлюються.
4. `git status && git diff` → review → commit як `docs: <короткий-summary>`.

`docs:regen` ніколи не модифікує drafts і не запускає normalize.

## Залежності і структура коду

### `scripts/docs-regen.js`

ESM-модуль, ціль — читабельність за 5 хвилин. Без TypeScript, без zod, без фреймворків.

Залежності:

- **Bun built-in:** `Bun.file`, `Bun.write`, `Bun.spawn` (для виклику CLI `claude`/`cursor-agent`).
- **Node стандарт:** `fs/promises`, `path`, `node:crypto` (sha256), `node:url`.
- **`gray-matter`** — парсити YAML frontmatter, відсіяти ADR з `session:`.
- **`globby`** — glob по `docs/adr/**/*.md` з ignore.

Структура файлу:

```js
// scripts/docs-regen.js
import { globby } from 'globby'
import matter from 'gray-matter'
import { createHash } from 'node:crypto'
import { readFile } from 'node:fs/promises'

const PROJECTIONS = ['01-context', '02-containers', '03-components', '04-code', 'decisions']

async function main(args) {
  // 1. parse CLI args
  // 2. acquire lock
  // 3. discover clean ADRs (with has_mark per ADR)
  // 4. load manifest
  // 5. detect triggers (unmarked ADRs, removed ADRs, templates/rule hash drift)
  // 6. if any trigger → regenerate all 5 projections: build inputs → call LLM → write file
  // 7. update marks in ADRs based on aggregated used_adrs
  // 8. write manifest
  // 9. log summary
}
```

Функції розбиваються логічно (`discoverCleanAdrs`, `loadManifest`, `diffAgainstManifest`, `regenerateProjection`, `updateAdrMark`, `callLlm`, `writeManifest`) — кожна без скритих залежностей, тестована окремо.

### `package.json`

Додається один скрипт:

```json
"docs:regen": "bun run scripts/docs-regen.js"
```

### `.gitignore`

Додається:

```
docs/ci4/.regen.log
.claude/hooks/.docs-regen.lock
```

### `.cursor/skills/docs-regen/SKILL.md`

Створюється новий skill-файл за зразком `.cursor/skills/n-adr-normalize/SKILL.md`. Опис, тригери, команда `bun run docs:regen`.

### `AGENTS.md` і `CLAUDE.md`

Регенеруються через `npx @nitra/cursor` після додавання нового skill — це частина imp плану, не цієї спеки.

## Перший запуск і робота з існуючим вмістом

Станом на 2026-05-18 у `docs/ci4/` уже є рукотворні `01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`. Перший запуск **переписує їх**.

### Підхід без `--bootstrap`

Перший запуск — `bun run docs:regen` (без додаткових прапорців). Алгоритм:

1. Manifest відсутній → trigger «regen all».
2. Для кожної проекції — LLM отримує `docs/ci4/<name>.md` як «попередній стан» і інструкцію «зберігай структуру і ручні формулювання, де вони мають сенс; розширюй за рахунок ADR; не видаляй раніше задокументоване, що не суперечить ADR».
3. LLM повертає новий `content`, регенератор пише поверх.

`git status && git diff` — єдине review-вікно. Розробник дивиться, приймає або відкатує (`git checkout -- docs/ci4/<name>.md`).

### Sanity-check перед першим запуском

Регенератор друкує сухий звіт у stdout перед стартом LLM-викликів:

```
docs-regen v0.1.0
- Clean ADRs found: 26
  - ADR-0006-google-oauth
  - adr-нормалізація-розмір-батчу-batch5
  - ...
- Drafts ignored: 0 (in docs/adr/), 0 (in docs/adr/_inbox/)
- Manifest: not found → full regen of all 5 projections
- Existing docs/ci4/ files: 01-context.md, 02-containers.md, 03-components.md, 04-code.md, decisions.md (will be overwritten)
- Templates: 6 files in docs/ci4/_templates/ (will be created if missing)
Press Ctrl-C within 3s to abort, otherwise proceeding...
```

3-секундна пауза — захист від помилкового запуску в забрудненому репо. Скрипт не питає інтерактивно (`y/n`), бо інтерактивність ускладнює запуск з агентського контексту.

### Bootstrap шаблонів

Якщо `docs/ci4/_templates/` відсутній — регенератор створює стартові шаблони з вбудованих в скрипт констант. Це гарантує, що **перший запуск працює з порожнього стану** без додаткових кроків. Розробник може потім редагувати шаблони, і їхні хеші стануть джерелом тригера «regen all» на наступному запуску.

### Drafts і `_inbox/`

Поза scope регенератора. Чекають `n-adr` normalize-flow. Якщо drafts накопичаться — `/n-adr-normalize` → переходить в clean → наступний `docs:regen` бере їх в inputs.

## Edge cases

| Сценарій | Поведінка |
| --- | --- |
| LLM повернув не-JSON / битий JSON | Retry 1 раз. Другий провал → exit 1, проекція не оновлюється, manifest не чіпається. |
| LLM повернув `used_adrs` з невідомим slug | Warn у лог, content приймаємо. Невідомий slug у мітку не йде. |
| ADR має preexisting мітку, LLM не використав цього разу | Мітка переписується на «Проекції: жодної.» |
| `docs/ci4/<name>.md` не існує (перший запуск) | LLM створює з нуля. `current_content` у inputs — порожній рядок. |
| `_templates/<name>.prompt.md` відсутній | Регенератор створює зі вбудованих констант. Логує `templates bootstrapped: <list>`. |
| Жодного clean ADR | Кожна проекція отримує тіло `> Документація не може бути регенерована: відсутні clean ADR у docs/adr/`. |
| ADR slug collision | Exit 2 з повідомленням `ADR slug collision: <slug> (paths: ...)`. Розробник вирішує руками. |
| Битий manifest (невалідний JSON) | Exit 2. Розробник видаляє руками — наступний запуск стартує з порожнього. |
| Жодного LLM CLI (`claude`, `cursor-agent`) у PATH | Exit 1 з повідомленням «No LLM CLI available». |
| Конкурентний запуск (lock зайнятий) | Exit 0 з повідомленням «Another docs:regen is running». |
| Репо в `MERGE_HEAD` / `rebase` | Exit 0 з повідомленням «Repository is in merge/rebase state, aborting». |

### Обмеження по токенах

Усі 26 clean ADR одним пакетом займають ~30-40k токенів (середній розмір ADR ~3 КБ × 26 = 78 КБ ≈ 20k токенів body + frontmatter). Це далеко від ліміту 200k Claude Sonnet, тож двофазний відбір (топ-N) не потрібний у Phase 1. Якщо в майбутньому ADR-ів стане 100+ — додамо ранжування окремою задачею.

## Тестування

### Юніт-тести

`scripts/__tests__/docs-regen/` (за зразком `app/src/services/*.test.js` через `bun:test`, але поза workspace `app/`, бо скрипт живе на root-рівні):

- `discoverCleanAdrs.test.js` — фікстури з 3 ADR (1 clean, 1 з `session:`, 1 у `_inbox/`) → повертає 1 slug.
- `detectTriggers.test.js` — фікстури з ADR (з міткою / без мітки), видаленими ADR з manifest, зміненими шаблонами → коректний список тригерів.
- `updateAdrMark.test.js` — кейси: «без мітки → додати», «з міткою → переписати», «без мітки, з horizontal rule в кінці body» (не плутаємо з sentinel), «мітка з 0 проекцій».
- `hasMark.test.js` — фікстури ADR з різним розташуванням `---` і `**Опрацьовано**`: правильний детект останнього sentinel-блоку як мітки.
- `parseLlmResponse.test.js` — валідний JSON, JSON у markdown fence, битий JSON, відсутні поля.

LLM-виклики **не** мокаються в інтеграції — фокус на детерміністичній частині (дискавері, diff, мітка, manifest, парсинг).

### Інтеграційний smoke-тест

Manual checklist для першого запуску:

1. `git status` — clean.
2. `bun run docs:regen --dry` → sanity-звіт без записів.
3. `bun run docs:regen` → 5 файлів у `docs/ci4/` оновлено, 26 ADR мають мітку, manifest створено.
4. `git diff docs/ci4/` — переглянути зміни кожного файлу.
5. `git diff docs/adr/` — переглянути мітки.
6. `bun run docs:regen` (повторно одразу) → exit `OK, nothing to regenerate`.
7. Тригер drift: видалити мітку `**Опрацьовано**` з одного ADR → `bun run docs:regen --check` → exit 1.
8. `bun run docs:regen --projection 01-context` → лише `01-context.md` змінений.

### Lint

- `bunx markdownlint-cli2 docs/ci4/**/*.md` — регенеровані файли проходять без warning'ів.
- `bun run lint-text` (n-cursor lint-text) — теж проходить.
- `npx @nitra/cursor check ci4` — поточна перевірка C4 не ламається.

## Verification checklist

Перед PR з реалізацією Phase 1:

- [ ] `bun run docs:regen` на чистому репо створює `docs/ci4/01..04.md`, `decisions.md`, `_templates/*.prompt.md`, `manifest.json` без помилок.
- [ ] Усі 26 clean ADR мають sentinel-блок «Опрацьовано» в кінці.
- [ ] `manifest.json` містить 26 ADR (з `processed_at` і `projections`), 5 проекцій (з `output_hash` і `used_adrs`), хеші шаблонів і правила `n-ci4.mdc`. Поля `hash` у `adrs` немає.
- [ ] `bun run docs:regen` без змін у репо — exit 0 з повідомленням `nothing to regenerate`.
- [ ] Видалення мітки з одного ADR + `bun run docs:regen --check` → exit 1.
- [ ] `bun run docs:regen --projection 01-context` оновлює тільки `01-context.md`.
- [ ] `bunx markdownlint-cli2 docs/ci4/**/*.md` проходить.
- [ ] `bun run lint-text` проходить.
- [ ] `bun run lint-js` проходить (новий `scripts/docs-regen.js`).
- [ ] Жодного HTML-тегу в регенерованих файлах і ADR-мітках.
- [ ] `.cursor/skills/docs-regen/SKILL.md` створено, `npx @nitra/cursor` синхронізує `AGENTS.md` і `CLAUDE.md`.

## Phase 2 (поза цією спекою)

Окрема спека опише `/docs-propose` workflow:

1. Інженер/менеджер каже LLM: «хочу змінити X».
2. LLM знаходить релевантні ADR (тут — за tags/components або через семантичний пошук).
3. LLM формулює draft ADR з контекстом і варіантами.
4. LLM показує impact: які проекції зміняться, як саме.
5. Користувач затверджує/править draft → status: accepted → файл у `docs/adr/`.
6. Регенератор (`bun run docs:regen`) запускається автоматично з `/docs-propose` після затвердження.
7. Git commit з ADR + регенерованими docs.

Phase 2 будується **поверх** Phase 1 (regenerator + manifest + мітки залишаються незмінними).
