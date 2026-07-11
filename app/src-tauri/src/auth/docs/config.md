---
type: Rust Module
title: config.rs
resource: app/src-tauri/src/auth/config.rs
docgen:
  crc: 03e33a6b
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.98
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Потрібно оновити секцію overview, тож спершу перегляну релевантні правила про документацію й changelog, а потім перепишу текст.Відкриваю правила документації та changelog.

## Поведінка

- desktop_client_id — повертає Google OAuth desktop client ID, використовуючи cached значення; якщо конфігурація відсутня, дає fallback-значення.
- desktop_client_secret — повертає Google OAuth desktop client secret для desktop-вхідного потоку; якщо секрет не заданий або не схожий на реальний, використовує вбудований fallback.
- android_client_id — повертає Google OAuth Android client ID з cached значенням і fallback-резервом.
- android_web_client_id — повертає Google OAuth Android web client ID з cached значенням і fallback-резервом.
- is_real_client_id — визначає, чи виглядає значення як реальний client ID; порожні, пробільні та fallback-значення не вважає реальними.

## Публічний API

- `desktop_client_id` — повертає desktop client ID з кешу; не кидає винятків назовні; за помилок повертає `null`
- `desktop_client_secret` — повертає desktop client secret з кешу; не кидає винятків назовні; за помилок повертає `null`
- `android_client_id` — повертає Android client ID з кешу; не кидає винятків назовні; за помилок повертає `null`
- `android_web_client_id` — повертає Android web client ID з кешу; не кидає винятків назовні; за помилок повертає `null`
- `is_real_client_id` — визначає, чи значення є справжнім client ID; не кидає винятків назовні; за помилок повертає `null`

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
- Перехоплює помилки і не пропускає винятків назовні (fail-safe).
- За певних помилок повертає порожнє значення (напр. `null`) замість винятку.
- Кешує результати в межах одного прогону.
