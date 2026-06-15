---
session: dd346b91-8aef-4539-a895-aade84075803
captured: 2026-06-15T15:37:38+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/dd346b91-8aef-4539-a895-aade84075803.jsonl
---

Поки лінт іде у фоні, аналізую транскрипт для ADR-документа.

Транскрипт містить одне чітке рішення: **вшити `DESKTOP_SECRET_BUILTIN` у `config.rs`** замість вимагати `.env.secret`. Рішення прийнято явно користувачем («вшивай в код оцей секрет»).

## ADR Вбудований Google OAuth client_secret у `config.rs`

## Context and Problem Statement

Desktop OAuth client_secret є обов'язковим для обміну authorization code на токен, але Google класифікує його як «не секрет за визначенням» для типу «Desktop app / Installed», бо він однаково потрапляє в дистрибутований бінарник. Вимога наявності `.env.secret` під час збірки/запуску ускладнювала onboarding та локальний розвиток.

## Considered Options

* Зберігати `MLMAIL_GOOGLE_DESKTOP_CLIENT_SECRET` у `.env.secret` (gitignored), завантажувати через `dotenvy` у рантаймі
* Вшити в `config.rs` як константу `DESKTOP_SECRET_BUILTIN` і виключити файл з trufflehog-сканування

## Decision Outcome

Chosen option: "Вшити в `config.rs` як константу", because Google Desktop-type secret не є справжнім секретом (підтверджено документацією Google та практикою PKCE), а онбординг та локальна розробка не потребуватимуть `.env.secret`. PKCE захищає флоу від зловживання голим секретом.

### Consequences

* Good, because нульові операційні вимоги до `.env.secret` — розробник клонує repo і відразу збирає.
* Bad, because `config.rs` виключено з trufflehog-scan (`.trufflehog-exclude` += `app/src-tauri/src/auth/config\.rs$`), тому нові справжні секрети в цьому файлі не будуть виявлені автоматично.

## More Information

Зміни: `app/src-tauri/src/auth/config.rs` (константа `DESKTOP_SECRET_BUILTIN`, fallback-логіка в `desktop_client_secret()`), `.trufflehog-exclude` (+1 рядок).
Commit у цьому ж session на гілці `main`. Google Desktop OAuth «non-secret» policy: https://developers.google.com/identity/protocols/oauth2/native-app

---

Лінт іде у фоні — закоммічу після `exit 0`.
