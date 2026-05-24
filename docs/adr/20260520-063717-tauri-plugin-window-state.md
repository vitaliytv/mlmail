# tauri-plugin-window-state: Збереження Геометрії Вікна між Запусками

**Status:** Accepted
**Date:** 2026-05-20

## Context and Problem Statement

Десктоп-застосунок на Tauri v2 щоразу відкривається зі стандартним розміром вікна, ігноруючи ручну зміну розміру попереднього сеансу. Потрібне автоматичне відновлення геометрії між запусками.

## Considered Options

- `tauri-plugin-window-state` — офіційний Tauri-плагін, зберігає розмір/позицію у файлі між сесіями
- `"maximized": true` у `tauri.conf.json` — завжди відкривати максимізованим, без збереження кастомного розміру
- Rust-код на старті, що читає висоту монітора — для «розтягнути тільки по висоті»

## Decision Outcome

Chosen option: "`tauri-plugin-window-state`", because це єдиний варіант, що зберігає саме той розмір, який виставив користувач, а не фіксований максимум; конфігурація порівнянна за простотою з варіантом `maximized`.

Реєстрацію `.plugin()` у `lib.rs` огорнуто в `#[cfg(desktop)]`, бо плагін має `#![cfg(not(any(target_os = "android", target_os = "ios")))]` і android-збірка ламається без цього захисту. `"window-state:default"` у capabilities видалено для android-сумісності.

### Consequences

- Good, because `cargo check` десктоп-збірки завершився без помилок (`Finished dev profile`); плагін працює суто на рівні Rust — JS-код не потрібен.
- Bad, because `"window-state:default"` у capabilities несумісний з android і вимагає явного видалення; реєстрацію `.plugin()` треба огортати `#[cfg(desktop)]`.
- Neutral, because поведінку (зміна розміру → закрити → перезапуск) не перевіряли автоматично — лише `cargo check`.

## More Information

Змінені файли:

- `app/src-tauri/Cargo.toml` — `tauri-plugin-window-state = "2"` у секції `[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]`
- `app/src-tauri/src/lib.rs` — `.plugin(tauri_plugin_window_state::Builder::default().build())` під `#[cfg(desktop)]`
- `app/src-tauri/capabilities/default.json` — `"window-state:default"` видалено з `permissions`

Версія у `Cargo.lock`: `tauri-plugin-window-state 2.4.1`. Стан вікна зберігається у `~/Library/Application Support/com.vitaliytv.mlmail/window-state.json` на macOS.
