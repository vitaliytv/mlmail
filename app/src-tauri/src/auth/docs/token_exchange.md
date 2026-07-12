---
type: Rust Module
title: token_exchange.rs
resource: app/src-tauri/src/auth/token_exchange.rs
docgen:
  crc: 64e1df71
  model: openai-codex/gpt-5.5
  score: 100
  issues: judge:inaccurate:0.99
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Файл забезпечує обмін OAuth authorization code і refresh token на токени доступу через Google token endpoint або заданий endpoint, зокрема `https://oauth2.googleapis.com/token`, `http://127.0.0.1:1234/callback`, `http://127.0.0.1:0/callback` і `http://r`. Він існує як fail-safe шар мережевої взаємодії з OAuth: повертає контрольовані результати для OAuth- та мережевих помилок, не кидає винятків назовні і за певних помилок повертає порожнє значення замість винятку.

## Поведінка

- `TokenResponse` представляє відповідь OAuth із токеном доступу, часом дії та опційними додатковими токенами, які можуть бути відсутніми.
- `FlowKind` розрізняє desktop- і Android-сценарії обміну коду, щоб надсилати лише потрібні OAuth-дані.
- `exchange_code` обмінює authorization code на токени через `https://oauth2.googleapis.com/token`; у desktop-сценарії очікуваний callback може бути на кшталт `http://127.0.0.1:1234/callback`.
- `exchange_refresh` оновлює access token через Google token endpoint і повертає вимогу повторної авторизації для недійсного refresh token.
- `exchange_code_at` виконує той самий обмін коду на токени для заданого endpoint, зокрема для тестових redirect URI на кшталт `http://127.0.0.1:0/callback`; мережеві й OAuth-помилки повертає як контрольований результат.
- `exchange_refresh_at` оновлює токени через заданий endpoint, підтримує повернення нового refresh token, якщо сервіс його надав, і класифікує помилки без винятків назовні, зокрема для endpoint на кшталт `http://r`.

## Публічний API

- TokenResponse — представляє результат відповіді token endpoint, зокрема `https://oauth2.googleapis.com/token`, щоб передати дані OAuth-обміну далі без запису у ФС чи БД.
- FlowKind — розрізняє сценарії OAuth-обміну для callback на `http://127.0.0.1:1234/callback`, `http://127.0.0.1:0/callback` або тестового префікса `http://r`.
- exchange_code — звертається до мережі й обмінює authorization code через `https://oauth2.googleapis.com/token`; працює fail-safe: перехоплює помилки, не кидає винятків назовні і за певних помилок повертає порожнє значення.
- exchange_refresh — звертається до мережі й виконує refresh-обмін через `https://oauth2.googleapis.com/token`; працює fail-safe: перехоплює помилки, не кидає винятків назовні і за певних помилок повертає порожнє значення.
- exchange_code_at — звертається до мережі й обмінює authorization code через явно заданий token endpoint; працює fail-safe: перехоплює помилки, не кидає винятків назовні і за певних помилок повертає порожнє значення.
- exchange_refresh_at — звертається до мережі й виконує refresh-обмін через явно заданий token endpoint; працює fail-safe: перехоплює помилки, не кидає винятків назовні і за певних помилок повертає порожнє значення.

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
- Перехоплює помилки і не пропускає винятків назовні (fail-safe).
- За певних помилок повертає порожнє значення (напр. `null`) замість винятку.
