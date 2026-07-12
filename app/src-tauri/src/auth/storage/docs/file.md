---
type: Rust Module
title: file.rs
resource: app/src-tauri/src/auth/storage/file.rs
docgen:
  crc: be5085f0
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.98
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

`FileStorage` створює й відкриває файлове сховище для сесії, а `new` ініціалізує його з конфіга `session.json`. Компонент працює fail-safe: помилки не виходять назовні, а за окремих збоїв повертається порожнє значення, наприклад `null`, замість винятку.

## Поведінка

- `FileStorage` — зберігає та відновлює дані сесії у файлі `session.json`, повертаючи порожнє значення, якщо запис ще відсутній; помилки обробляє fail-safe як backend-помилки.
- `new` — створює файлове сховище для сесії на основі переданого шляху до `session.json`.

## Публічний API

- FileStorage — зберігає та відновлює дані, спираючись на `session.json`
- new — створює новий стан сховища

## Гарантії поведінки

- Перехоплює помилки і не пропускає винятків назовні (fail-safe).
- За певних помилок повертає порожнє значення (напр. `null`) замість винятку.
