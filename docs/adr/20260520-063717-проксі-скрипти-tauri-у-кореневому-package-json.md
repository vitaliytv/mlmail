---
session: 04fe87d3-2173-4c1b-af9f-07e63c0d0afb
captured: 2026-05-20T06:37:17+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/04fe87d3-2173-4c1b-af9f-07e63c0d0afb.jsonl
---

## ADR Проксі-скрипти Tauri у кореневому `package.json`

## Context and Problem Statement

Єдиний репозиторій має workspace `app`, де знаходяться скрипти Tauri. Запустити `tauri dev` чи `tauri build` з кореня неможливо без явної зміни директорії, що незручно при роботі з кореневим `package.json`.

## Considered Options

- `cd app && bun run tauri <cmd>` — зміна директорії через shell
- `bun --cwd=app run tauri <cmd>` — прапорець `bun` без зміни CWD оболонки

## Decision Outcome

Chosen option: "`bun --cwd=app run tauri <cmd>`", because флаг `--cwd` змінює лише внутрішній CWD процесу bun, не зачіпаючи середовище оболонки; перевірено `bun --cwd=app run tauri --version` → `tauri-cli 2.11.1`.

### Consequences

- Good, because transcript фіксує очікувану користь: `tauri-cli 2.11.1` коректно резолвиться з кореня без `cd`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінено `/Users/vitaliytv/www/vitaliytv/mlmail/package.json`, скрипти `dev` і `tauri`:

```json
"dev": "bun --cwd=app run tauri dev",
"tauri": "bun --cwd=app run tauri"
```

---

## ADR Збереження розміру вікна через `tauri-plugin-window-state` з desktop-only поетапним вмиканням

## Context and Problem Statement

Десктоп-застосунок (macOS) щоразу відкривається зі стандартним розміром вікна, ігноруючи попередню зміну розміру користувачем. Потрібно або запам'ятовувати розмір між запусками, або завжди відкривати максимізованим.

## Considered Options

- `tauri-plugin-window-state` — плагін, що зберігає розмір і позицію вікна при закритті та відновлює при старті
- `"maximized": true` у `tauri.conf.json` — завжди відкривати максимізованим, без збереження кастомного розміру
- Rust-код на старті, що читає висоту монітора — для «розтягнути тільки по висоті»

## Decision Outcome

Chosen option: "`tauri-plugin-window-state`", because це єдиний варіант, що зберігає саме той розмір, який виставив користувач, а не фіксований максимум; конфігурація порівнянна за простотою з варіантом `maximized`.

### Consequences

- Good, because transcript фіксує очікувану користь: `cargo check` десктоп-збірки завершився без помилок (`Finished dev profile`); плагін працює суто на рівні Rust — JS-код не потрібен.
- Bad, because плагін має `#![cfg(not(any(target_os = "android", target_os = "ios")))]`, тому реєстрацію `.plugin()` у `lib.rs` потрібно огорнути `#[cfg(desktop)]`, інакше android-збірка ламається; `window-state:default` у capabilities несумісний з android і був видалений.

## More Information

Змінені файли:

- `app/src-tauri/Cargo.toml` — `tauri-plugin-window-state = "2"` перенесено до секції `[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]`
- `app/src-tauri/src/lib.rs` — реєстрація розширення під `#[cfg(desktop)]`
- `app/src-tauri/capabilities/default.json` — прибрано `"window-state:default"` з `permissions`

Версія розширення у `Cargo.lock`: `tauri-plugin-window-state 2.4.1`. Поведінку (зміна розміру → закрити → перезапуск) не перевіряли автоматично — лише `cargo check`.
