---
session: 04fe87d3-2173-4c1b-af9f-07e63c0d0afb
captured: 2026-05-20T05:34:34+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/04fe87d3-2173-4c1b-af9f-07e63c0d0afb.jsonl
---

## ADR Проксі-скрипти Tauri у кореневому package.json

## Context and Problem Statement
Проєкт має монорепо-структуру: `app/` — підпакет з Tauri-застосунком. Щоб запускати `tauri dev` і загальні tauri-команди з кореня репо без ручного переходу в `app/`, потрібні проксі-скрипти в кореневому `package.json`.

## Considered Options
* `cd app && bun run tauri ...` у скриптах кореневого `package.json`
* `bun --cwd=app run tauri ...` у скриптах кореневого `package.json`

## Decision Outcome
Chosen option: "`bun --cwd=app run tauri ...`", because флаг `--cwd` не змінює поточну директорію шела (на відміну від `cd`), і підтвердився тестом `bun --cwd=app run tauri --version` → `tauri-cli 2.11.1` без жодного `cd`.

### Consequences
* Good, because кореневий `bun run dev` і `bun run tauri` резолвяться без зміни CWD шела — `tauri-cli 2.11.1` відповідає коректно.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Змінений файл: `package.json` (кореневий).
Додані скрипти:
```json
"dev":   "bun --cwd=app run tauri dev",
"tauri": "bun --cwd=app run tauri"
```
Команда верифікації: `bun run tauri --version` → `$ bun --cwd=app run tauri --version` → `tauri-cli 2.11.1`.
Bun версії `1.3.14`; підтримує `--cwd=<val>` з відносним шляхом.
