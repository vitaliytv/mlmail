---
session: 04fe87d3-2173-4c1b-af9f-07e63c0d0afb
captured: 2026-05-20T05:41:23+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/04fe87d3-2173-4c1b-af9f-07e63c0d0afb.jsonl
---

Крейт завантажено. Зміни:

| Файл | Що змінено |
|---|---|
| `Cargo.toml` | `tauri-plugin-window-state = "2"` |
| `src/lib.rs` | `.plugin(tauri_plugin_window_state::Builder::default().build())` |
| `capabilities/default.json` | `"window-state:default"` у permissions |

Після першого `bun run dev` вручну змініть розмір вікна — наступні запуски відновлять його автоматично. Стан зберігається у `~/.local/share/mlmail/window-state.json` (macOS: `~/Library/Application Support/com.vitaliytv.mlmail/window-state.json`).

---

## ADR Проксі-скрипти Tauri у кореневому package.json

## Context and Problem Statement
Проєкт має monorepo-структуру з кореневим `package.json` і вкладеним `app/package.json`. Команди `tauri dev` і `tauri build` були доступні лише з директорії `app/`, що вимагало ручного `cd` перед кожним запуском.

## Considered Options
* Проксі через `cd app && bun run tauri …`
* Проксі через `bun --cwd=app run tauri …`

## Decision Outcome
Chosen option: "Проксі через `bun --cwd=app run tauri …`", because флаг `bun --cwd` дозволяє запустити скрипт у піддиректорії без зміни поточної директорії шелу; тест `bun --cwd=app run tauri --version` підтвердив коректну відповідь `tauri-cli 2.11.1` без `cd`.

### Consequences
* Good, because кореневий `bun run dev` і `bun run tauri` тепер проксіюють в `app/` без зміни CWD шелу.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Змінено `scripts.dev` і `scripts.tauri` у `/Users/vitaliytv/www/vitaliytv/mlmail/package.json`. Верифіковано командою `bun run tauri --version` з кореня.

---

## ADR tauri-plugin-window-state для збереження геометрії вікна

## Context and Problem Statement
Десктоп-застосунок на Tauri v2 щоразу відкривається з фіксованим розміром `800×600` (з `tauri.conf.json`), ігноруючи ручний ресайз попереднього сеансу. Потрібне автоматичне відновлення геометрії між запусками.

## Considered Options
* `tauri-plugin-window-state` — офіційний Tauri-плагін, зберігає розмір/позицію у файлі між сесіями
* Фіксований розмір у `tauri.conf.json` (хардкод під конкретний дисплей)

## Decision Outcome
Chosen option: "`tauri-plugin-window-state`", because плагін автоматично запам'ятовує геометрію після першого ручного ресайзу і відновлює її при наступному запуску без додаткового коду.

### Consequences
* Good, because transcript фіксує очікувану користь: вікно відновлює розмір і позицію між сесіями без будь-яких змін у Vue/JS коді.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Змінені файли:
- `app/src-tauri/Cargo.toml` — додано `tauri-plugin-window-state = "2"`
- `app/src-tauri/src/lib.rs` — `.plugin(tauri_plugin_window_state::Builder::default().build())`
- `app/src-tauri/capabilities/default.json` — `"window-state:default"` у `permissions`

Стан вікна зберігається у `~/Library/Application Support/com.vitaliytv.mlmail/window-state.json` на macOS. Крейт завантажено через `cargo fetch`.
