# Tauri Managed State для Storage та Endpoints (DI для тестованості)

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

Команди `auth_logout`, `acquire_access_token` та gmail-команди мали зашиті залежності: keyring-storage через `#[cfg(target_os)]` та константи Google URLs. Без можливості підміни цих залежностей неможливо тестувати будь-який шлях, що зачіпає keyring або зовнішній HTTP, без реального Keychain і мережі.

## Considered Options

- `#[cfg(test)]` підміна залежностей безпосередньо в prod-коді
- Environment variable з `OnceLock` для перевизначення URL у runtime
- Tauri managed state як механізм DI для команд

## Decision Outcome

Chosen option: "Tauri managed state", because це стандартний механізм DI для Tauri-команд; перехід до `Arc` є обов'язковим, оскільки `State<>` вимагає `Send + Sync + 'static`.

### Consequences

- Good, because тести можуть підставляти `InMemoryStorage` замість реального Keychain без змін у prod-коді.
- Good, because `mockito`-сервер підставляється як `Endpoints` замість реальних Google URLs.
- Bad, because перехід з `Box<dyn RefreshTokenStorage>` на `Arc<dyn RefreshTokenStorage>` потребував рефакторингу сигнатур і реєстрації в `lib.rs`.
- Neutral, because публічний API Tauri-команд не змінився — `State<>` залишається незмінним.

## More Information

Конкретні зміни:
- `Box<dyn RefreshTokenStorage>` → `Arc<dyn RefreshTokenStorage>` під аліасом `SharedStorage`; реєструється через `.manage()` у `lib.rs::run()`.
- Новий файл `src-tauri/src/endpoints.rs`: `Endpoints { google_token, gmail_label_inbox, gmail_messages_list }` з `impl Default` (реальні Google URLs); реєструється через `.manage(Endpoints::default())`.
- `acquire_access_token` і `finalize_login` тепер приймають URL-рядок і `&dyn RefreshTokenStorage` явно, не беруть їх зсередини.

Зачіпає: `src-tauri/src/auth/storage/mod.rs` (SharedStorage), `src-tauri/src/endpoints.rs` (новий), `src-tauri/src/auth/mod.rs`, `src-tauri/src/gmail/mod.rs`, `src-tauri/src/lib.rs`.

---

**Опрацьовано** 2026-05-20. Проекції:
- [01-context](../ci4/01-context.md)
- [02-containers](../ci4/02-containers.md)
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
