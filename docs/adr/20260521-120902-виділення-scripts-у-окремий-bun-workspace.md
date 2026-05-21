---
session: ecf97f5f-d56c-48f6-8917-b4e114439bc2
captured: 2026-05-21T12:09:02+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/ecf97f5f-d56c-48f6-8917-b4e114439bc2.jsonl
---

## ADR Виділення `scripts/` у окремий Bun workspace

## Context and Problem Statement
Правило `n-bun.mdc` забороняє розміщувати залежності конкретних інструментів у кореневих `devDependencies`. `globby` і `gray-matter`, що використовуються лише скриптами `scripts/docs-regen`, знаходились у кореневому `package.json`, що порушувало перевірку `npx @nitra/cursor check`.

## Considered Options
* Виділити `scripts/` в окремий Bun workspace із власним `package.json`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Виділити `scripts/` в окремий Bun workspace", because правило `n-bun.mdc` вимагає, щоб залежності інструментів жили у відповідному workspace, а не в кореневих `devDependencies`; `npx @nitra/cursor check` повернув `bun: ❌` саме через це.

### Consequences
* Good, because transcript фіксує очікувану користь: `bun: ✅` після переміщення; кореневий `package.json` більше не містить залежностей, що стосуються лише `scripts/`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Створено `scripts/package.json` з `gray-matter ^4.0.3` і `tinyglobby ^0.2.16`; у кореневий `package.json` додано `"scripts"` до масиву `workspaces`; виконано `bun i` — встановлено 1 пакет, видалено 2.

---

## ADR Заміна `gitleaks` на `trufflehog` для `lint-security`

## Context and Problem Statement
Правило `n-security.mdc` канонізує `trufflehog filesystem` як інструмент локального та CI-безпекового лінту. Скрипт `lint-security` у `package.json` використовував `gitleaks detect`, що не відповідало канону правила; перевірка `npx @nitra/cursor check` фіксувала `security: ❌`.

## Considered Options
* Замінити `gitleaks` на `trufflehog filesystem … --exclude-paths .trufflehog-exclude --results=verified,unknown --fail`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Замінити `gitleaks` на `trufflehog filesystem`", because `n-security.mdc` явно визначає `trufflehog` як канонічний інструмент; відповідність перевірці `npx @nitra/cursor check` є обов'язковою умовою.

### Consequences
* Good, because transcript фіксує очікувану користь: `security: ✅` після заміни; створено `.trufflehog-exclude` і `.github/workflows/lint-security.yml`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Новий скрипт: `trufflehog filesystem . --no-update --exclude-paths .trufflehog-exclude --results=verified,unknown --fail`. Файл `.trufflehog-exclude` виключає `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures`. CI-файл `lint-security.yml` запускається на `push`/`pull_request` до `dev`/`main`.

---

## ADR Заміна `globby` на `tinyglobby` в `scripts/docs-regen/discover.js`

## Context and Problem Statement
Після виділення `scripts/` у workspace виявилося, що `globby` є ESM-пакетом без підтримки CommonJS; натомість `tinyglobby` вже був присутній у `node_modules` як транзитивна залежність і надає сумісний API (`glob` замість `globby`). Правило `@e18e/eslint-plugin` (підключене через `@nitra/eslint-config`) фіксувало `globby` як менш ефективну альтернативу.

## Considered Options
* Залишити `globby ^14.0.2` як пряму залежність у `scripts/package.json`
* Замінити на `tinyglobby ^0.2.16` із використанням `glob` замість `globby`

## Decision Outcome
Chosen option: "Замінити на `tinyglobby ^0.2.16`", because `tinyglobby` вже присутній у `node_modules` (транзитивна залежність), надає сумісний API і не отримує зауважень від лінтера `@e18e`.

### Consequences
* Good, because transcript фіксує очікувану користь: `eslint` без помилок після заміни; `bun i` встановив 2 нові пакети замість `globby`.
* Bad, because Neutral, because transcript не містить підтвердження наслідку щодо можливих розбіжностей поведінки між `globby` і `tinyglobby`.

## More Information
У `discover.js`: `import { globby } from 'globby'` → `import { glob } from 'tinyglobby'`; виклик `globby([…], opts)` → `glob([…], opts)`. Додатково `paths.sort()` → `paths.toSorted()` (вимога `unicorn/no-array-sort`).

---

## ADR Ігнорування навмисних дублікатів шаблонів у jscpd

## Context and Problem Statement
`jscpd` виявив 6 клонів: файли `scripts/docs-regen/default-templates/*.prompt.md` побайтово збігаються з `docs/ci4/_templates/*.prompt.md`. Це навмисне дублювання: функція `bootstrapTemplates()` копіює bundled-шаблони зі `scripts/` у директорію проєкту `docs/ci4/_templates/` при першому запуску. jscpd не знав про цей зв'язок і повертав `exit code 1`.

## Considered Options
* Виключити обидві директорії шаблонів з перевірки jscpd через `.jscpd.json`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Виключити обидві директорії шаблонів з `.jscpd.json`", because дублювання є свідомим контрактом архітектури: `bootstrapTemplates()` копіює `default-templates/` у `docs/ci4/_templates/` і обидві версії повинні бути ідентичними.

### Consequences
* Good, because transcript фіксує очікувану користь: jscpd більше не повертає помилок для цих файлів.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
До `.jscpd.json` додано `"scripts/docs-regen/default-templates/**"` і `"docs/ci4/_templates/**"` у масив `ignore`. Рішення підтверджено через `AskUserQuestion` — користувач підтвердив підхід.

---

## ADR Виключення `docs/ci4/manifest.json` з перевірки v8r

## Context and Problem Statement
`v8r` (JSON-валідація в `lint-text`) не міг завершитись успішно для `docs/ci4/manifest.json`: ім'я файлу `manifest.json` збігається з кількома несумісними схемами у Schema Store, і v8r повертав `Could not find a schema` / `Found multiple possible schemas`. Файл є власним маніфестом інструмента `docs-regen` і не має авторитетної схеми у Schema Store.

## Considered Options
* Додати `docs/ci4/manifest.json` до `.v8rignore`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Додати `docs/ci4/manifest.json` до `.v8rignore`", because файл є внутрішнім артефактом `docs-regen` без публічної JSON Schema; валідація через Schema Store семантично безглузда і блокує `lint`.

### Consequences
* Good, because transcript фіксує очікувану користь: `lint-text` виходить з кодом 0 після додавання до `.v8rignore`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Аналогічно до `.v8rignore` додано `.claude/settings.local.json` — з тієї ж причини (власний формат без Schema Store схеми). Файл `docs/ci4/manifest.json` також додано до `cspell ignorePaths` через схожий принцип.
