---
type: Rust Module
title: android.rs
resource: app/src-tauri/src/auth/flow/android.rs
docgen:
  crc: fd67fc41
  model: openai-codex/gpt-5.4-mini
  score: 100
  issues: judge:inaccurate:0.99
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Модуль надає публічні функції `run_login_flow`, `call_save_session`, `call_load_session`, `call_clear_session` для керування login flow і user session. `run_login_flow` узгоджено запитує доступи `openid`, `email`, `https://www.googleapis.com/auth/gmail.modify`, `https://www.googleapis.com/auth/gmail.settings.basic` і обмінює server auth code на token. `call_save_session` зберігає сесію, `call_load_session` читає її, `call_clear_session` очищає її. Модуль не працює напряму з ФС чи БД. Усі помилки обробляються fail-safe: винятки назовні не кидаються, а за певних помилок, зокрема під час читання, повертається порожнє значення.

## Поведінка

- `run_login_flow` — запускає Android sign-in через mobile plugin, запитує доступи `openid`, `email`, `https://www.googleapis.com/auth/gmail.modify`, `https://www.googleapis.com/auth/gmail.settings.basic`, обмінює отриманий server auth code на token і повертає результат або fail-safe помилку.
- `call_save_session` — зберігає сесію користувача через mobile plugin; не пише у ФС чи БД напряму і повертає помилку як текст, не кидаючи її назовні.
- `call_load_session` — читає збережену сесію через mobile plugin і повертає її, а якщо даних нема або вони неповні, повертає порожнє значення замість помилки.
- `call_clear_session` — очищає збережену сесію через mobile plugin; не пише у ФС чи БД напряму і повертає помилку як текст, не кидаючи її назовні.

## Публічний API

`api`

Забезпечує вхідний OAuth flow для доступу до Gmail і керування session-даними для подальших запитів.

Публічні точки входу:

- `run_login_flow` — ініціює авторизацію користувача для Gmail.
- `call_save_session` — зберігає session після успішного входу.
- `call_load_session` — відновлює раніше збережену session.
- `call_clear_session` — скидає збережену session і повертає стан до незалогіненого.

Гарантії:

- Працює у fail-safe режимі: помилки не виходять назовні як exception.
- За окремих збоїв повертає порожнє значення замість помилки.
- Не виконує запис у ФС або БД.

Доступи OAuth:

- <https://www.googleapis.com/auth/gmail.modify>
- <https://www.googleapis.com/auth/gmail.settings.basic>

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
- Перехоплює помилки і не пропускає винятків назовні (fail-safe).
- За певних помилок повертає порожнє значення (напр. `null`) замість винятку.
