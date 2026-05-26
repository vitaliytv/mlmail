---
name: n-coverage
description: >-
  Запустити `bun run coverage`, проаналізувати вижилих мутантів через LLM і зберегти
  результати в `reports/stryker/survived.md`
---

# n-coverage — аналіз покриття з LLM-рекомендаціями

## Мета

Запустити `bun run coverage`, зчитати вижилих мутантів Stryker, для кожного проаналізувати
причину виживання і зберегти `reports/stryker/survived.md`.

## Передумови

- CWD — корінь проєкту де є `package.json` зі скриптом `coverage`

## Workflow

### Step 1: Запуск coverage

```bash
bun run coverage
```

Очікуємо: `COVERAGE.md` оновлено, `reports/stryker/mutation.json` існує.
Якщо скрипт `coverage` відсутній у `package.json` → зупинись і повідом користувачу що потрібен скрипт `coverage` у `package.json`.

### Step 2: Зчитати вижилих мутантів

Прочитай `reports/stryker/mutation.json`. Шлях шукай відносно CWD:
- спочатку `reports/stryker/mutation.json`
- потім `app/reports/stryker/mutation.json` (монорепо з app/ підкаталогом)

Знайди всі мутанти де `status === "Survived"`. Ключ батьківського об'єкта у `files` — це шлях до файлу.
Кожен мутант містить: `mutatorName`, `replacement`, `location.start.line`.

Якщо survived порожній → виведи `✅ Score 100% — survived.md не потрібен` і зупинись.

### Step 3: LLM-аналіз кожного мутанта

Для кожного вижилого мутанта (обробляй по одному):

Спочатку визнач `jsRoot` — директорію де лежить `stryker.config.mjs`:
- Шукай `stryker.config.mjs` у CWD
- Потім у `app/stryker.config.mjs` (монорепо)
- `jsRoot` = директорія де знайдений файл (CWD або `app/`)

1. Прочитай source-файл. Шлях: `<jsRoot>/<file>`.
2. Витягни рядки від `(mutatedLine - 15)` до `(mutatedLine + 15)`.
   Якщо файл не знайдено за шляхом `jsRoot/file` → пропусти цей мутант і продовж з наступним.
3. Запам'ятай:
   - `originalCode` — вміст рядка `mutatedLine` у source файлі
   - `replacement` — поле з mutation.json (що Stryker підставив)
   - `mutatorName` — тип мутації
4. На основі прочитаного коду згенеруй:
   - **Причина виживання** (1-2 речення): чому жоден тест не виявив цю зміну?
   - **Що тестувати** (1-2 речення): конкретний сценарій, що вловить мутант.

### Step 4: Записати survived.md

Запиши `survived.md` у ту ж директорію де знайдено `mutation.json` (зафіксовано в Step 2).

Запиши `reports/stryker/survived.md` (або `app/reports/stryker/survived.md`):

```
# Survived Mutants

| Файл | Рядок | Тип мутації | Оригінал | Заміна Stryker | Причина виживання | Що тестувати |
| --- | --- | --- | --- | --- | --- | --- |
| src/i18n/auth-errors.js | 19 | ConditionalExpression | `kind !== null && kind !== undefined` | `false` | Тести не передають `null` як аргумент — умова завжди true у наявних тестах. | Додати тест де аргумент `null`, очікуючи fallback-значення. |
```

Сортуй рядки за файлом (алфавітно), потім за номером рядка.
