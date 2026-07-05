# Вбудований Google Desktop OAuth client_secret у вихідний код

**Status:** Accepted
**Date:** 2026-06-15

## Context and Problem Statement

macOS-логін завершувався з `AuthError::OAuth` через відсутній `.env.secret` — `desktop_client_secret()` повертала placeholder `REPLACE_ME_…`, що відхиляється Google при token-exchange. Зберігання лише у `.env.secret` (gitignored) вимагало ручного налаштування на кожному середовищі. Google явно зазначає, що `client_secret` для Desktop app є non-confidential: він однаково потрапляє у бінарник, а PKCE захищає флоу незалежно від секрету.

## Considered Options

* Вшити `DESKTOP_SECRET_BUILTIN` у `config.rs` як fallback; `.env.secret` лишається пріоритетним override
* Тримати secret виключно у `.env.secret` (gitignored)

## Decision Outcome

Chosen option: "Вшити `DESKTOP_SECRET_BUILTIN` у `config.rs`", because extractability з бінарника однакова в обох варіантах; вшивання усуває залежність від `.env.secret` і спрощує onboarding без втрати захищеності.

### Consequences

* Good, because `desktop_client_secret()` бере env/`.env.secret` якщо задано, інакше повертає built-in — `.env.secret` стає необов'язковим для desktop-розробника.
* Bad, because секрет у git-історії та в transcript — при витоку репо ротація обов'язкова через Google Cloud Console.
* Bad, because trufflehog без виключення блокував lint-security — додано `app/src-tauri/src/auth/config\.rs$` у `.trufflehog-exclude`.

## More Information

Файли: `app/src-tauri/src/auth/config.rs` (константа `DESKTOP_SECRET_BUILTIN`, `// cspell:disable-line`; функція `desktop_client_secret()` env → built-in fallback). `.trufflehog-exclude` += рядок-виняток. `trufflehog filesystem . --exclude-paths .trufflehog-exclude` → 0 findings. Коміт: `967482f`. `client_id` не вшивається — задається через `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID` у `.env`.
