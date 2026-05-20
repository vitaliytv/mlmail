# Файловий стор токенів замість macOS Keychain

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

При кожному запуску `tauri dev` macOS відображав системний промпт «Дозволити доступ до Keychain?», бо ad-hoc підпис бінарника змінювався між перебудовами, і ACL «Завжди дозволяти» прив'язувався до конкретного binary hash. Це систематично заважало розробці.

## Considered Options

- Файловий стор (`FileStorage`) — JSON `{app_data_dir}/session.json` з правами `0600`, завжди (dev + release).
- Release-збірка для щоденної роботи (підпис стабільний → ACL діє, але незручно як дефолт розробки).
- Ручне видалення/скидання ACL у Keychain Access після кожної перебудови (нестійко, потребує ручного кроку після кожної перебудови).
- `FileStorage` тільки для debug-збірки через `#[cfg(debug_assertions)]` — відхилено користувачем на користь повного переходу.

## Decision Outcome

Chosen option: "FileStorage завжди", because токен прив'язується до файлу, а не до ACL бінарника — промпт більше не з'являється ні в dev, ні в release. Користувач явно обрав «файловий стор завжди» замість умовної логіки.

### Consequences

- Good, because системний Keychain-промпт зникає повністю в dev та release.
- Good, because атомарний запис: temp-файл відкривається одразу з `mode(0o600)` через `OpenOptions::mode`, потім POSIX `rename` замінює цільовий — закриває TOCTOU-вразливість схеми write→chmod.
- Neutral, because після переходу потрібен одноразовий повторний логін (старий refresh-token у Keychain більше не читається).
- Bad, because transcript не містить підтвердження щодо security-аудиту зберігання токенів у файлі в release-середовищі.

## More Information

Зачіпає:
- `app/src-tauri/src/auth/storage/file.rs` — новий файл, `FileStorage`, 7 unit-тестів (всі зелені).
- `app/src-tauri/src/auth/storage/mod.rs` — `platform_storage()` на macOS повертає `FileStorage`; `pub mod macos` видалено.
- `app/src-tauri/src/auth/mod.rs` — `make_storage()` передає `app.path().app_data_dir().join("session.json")` у `FileStorage`.
- `app/src-tauri/src/auth/storage/macos.rs` — видалено повністю.
- `app/src-tauri/Cargo.toml` — залежність `keyring` (apple-native) видалена; `tempfile` додано до `[dev-dependencies]`.
- `app/src-tauri/Cargo.lock` — оновлено.

---

**Опрацьовано** 2026-05-20. Проекції:
- [01-context](../ci4/01-context.md)
- [02-containers](../ci4/02-containers.md)
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
