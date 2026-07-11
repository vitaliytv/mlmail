---
type: JS Module
title: login.js
resource: app/src/i18n/login.js
docgen:
  crc: a05a3124
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.98
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

`loginMessages` надає тексти для login-flow: повідомлення, які показує екран входу.

Read-only: модуль не пише у ФС чи БД.

АНКОРИ: `loginMessages`

## Поведінка

1. `loginMessages` надає набір текстів для екрана входу, щоб інтерфейс показував узгоджену назву застосунку.
2. Для ключа `appTitle` повертається брендова назва `MLMaiL`, яка використовується як публічне найменування продукту в login-flow.
3. Модуль працює лише на читання: нічого не записує у ФС чи БД, тому не створює побічних змін у даних.

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
