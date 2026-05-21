---
session: 04fe87d3-2173-4c1b-af9f-07e63c0d0afb
captured: 2026-05-20T05:33:10+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/04fe87d3-2173-4c1b-af9f-07e63c0d0afb.jsonl
---

## ADR Проксі-скрипти Tauri у кореневому `package.json`

## Context and Problem Statement

Проєкт `mlmail` має monorepo-структуру: кореневий `package.json` і workspace `app/`. Всі Tauri-скрипти (`tauri dev`, `tauri android dev`) визначені всередині `app/package.json`. Запуск `tauri dev` вимагав переходу в `app/` — з кореня прямого способу не було. Окрім того, потрібно розрізняти `mac` (desktop dev) та Android-запуск, щоб вони не конфліктували за аргументами.

## Considered Options

- Додати проксі-скрипти у кореневий `package.json` через `cd app && bun run ...`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Проксі-скрипти у кореневому `package.json`", because користувач явно вимагав запускати Tauri dev з кореня без переходу до `app/`, а `bun run --cwd` або `bun --filter` не розглядалися.

Два скрипти додані:

```json
"tauri": "cd app && bun run tauri",
"mac":   "cd app && bun run tauri dev",
```

`tauri` — загальний проксі для будь-яких Tauri-підкоманд (`tauri android dev` тощо); `mac` — фіксований аліас саме на `tauri dev` для mac-настільного застосунку.

### Consequences

- Good, because `bun run mac` і `bun run tauri <args>` працюють з кореня без ручного `cd app`.
- Bad, because `mac` і `android` (наявний в `app/package.json`) залишаються в різних місцях — `android` ще не обгорнутий проксі-функцією у корінь.

## More Information

- Змінений файл: `/Users/vitaliytv/www/vitaliytv/mlmail/package.json`, рядки `"tauri"` і `"mac"`.
- Наявний скрипт в `app/package.json`: `"android": "tauri android dev"` — не перенесено в корінь у цій сесії.
- Tauri CLI версія: `tauri-cli 2.11.1`, підтверджено командою `bun run tauri --version` з кореня.
