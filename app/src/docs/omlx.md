---
type: JS Module
title: omlx.js
resource: app/src/omlx.js
docgen:
  crc: e8227016
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.96
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Модуль створює OpenAI-compatible chat client для `omlx` і дає реактивний доступ до його конфігурації. Він використовує `response.json` як джерело налаштувань, звертається до мережі та працює з базовими адресами `http://127.0.0.1:8000/v1` і `http://127.0.0.1:8088/v1`. Публічні функції: `createOpenAiChat`, `useOmlx`. Кешування діє в межах прогону.

## Поведінка

- createOpenAiChat — створює чат-клієнт для OpenAI-compatible omlx endpoint, який відправляє completion-запит до локального сервера через `response.json` і повертає повідомлення асистента; підтримує базові URL `http://127.0.0.1:8000/v1` і `http://127.0.0.1:8088/v1` через зовнішню конфігурацію.
- useOmlx — надає реактивну omlx-конфігурацію з кешуванням у межах прогону, підтягує значення з локального сховища та середовища, а також автоматично вибирає доступний модельний id, коли явний model ще не заданий.

## Публічний API

- createOpenAiChat — Створює `chat` для запитів до OpenAI-compatible endpoint `omlx`, щоб прямий чат міг працювати через сумісний API.
- useOmlx — Зберігає налаштування локального `omlx`-server для direct-chat composables: `baseUrl` і `model` користувач змінює та бачить після перезапуску завдяки `localStorage`; `loadEnv` тимчасово підхоплює `myllm` proxy, поки він доступний, і спрямовує трафік через нього без запису в збережений конфіг; `storagePrefix` розділяє ключі `localStorage` між різними composables і фічами.

## Гарантії поведінки

- Кешує результати в межах одного прогону.
