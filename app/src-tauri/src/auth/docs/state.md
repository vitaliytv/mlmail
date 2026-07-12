---
type: Rust Module
title: state.rs
resource: app/src-tauri/src/auth/state.rs
docgen:
  crc: 51a645b9
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.94
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

`AuthState` зберігає поточний стан автентифікації користувача: email, access token і момент, після якого він вважається непридатним. Публічний API `is_access_token_fresh` дозволяє перевірити, чи access token ще дійсний, а `reset` — скинути стан автентифікації до початкового.

## Поведінка

- AuthState — зберігає поточний стан автентифікації: email, access token і момент його завершення.
- is_access_token_fresh — показує, чи є access token присутнім і достатньо далеким від завершення, щоб вважатися ще придатним.
- reset — очищає стан автентифікації та прибирає всі збережені значення.

## Публічний API

- AuthState — описує поточний стан авторизації для подальших дій
- is_access_token_fresh — перевіряє, чи access token ще вважається свіжим
- reset — повертає стан авторизації до вихідного значення

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
