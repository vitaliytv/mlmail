---
type: Rust Module
title: llm.rs
resource: app/src-tauri/src/llm.rs
docgen:
  crc: 2fbcc873
  model: manual
---

## Огляд

Надає Tauri-команди для явного підключення до одного OpenAI-compatible локального сервера на desktop-платформах.

## Поведінка

- Приймає лише HTTP(S) URL, що завершується на `/v1/`, і відхиляє некоректні адреси до мережевого виклику.
- Визнає один провайдер `local-openai`; початковий URL може бути переданий через `N_LOCAL_OPENAI_BASE_URL`.
- Повертає моделі та виконує one-shot chat за URL, переданим з UI; ключ із UI не записується на диск.
- Коли endpoint не налаштований, повертає зрозумілу помилку замість fallback на застарілий сервер чи конфіг.

## Публічний API

- `llm_default_config` — повертає не секретний стартовий URL із environment.
- `llm_providers` — повертає `local-openai` на desktop.
- `llm_list_models` — перевіряє endpoint і повертає його модельні id.
- `llm_chat` — надсилає один системний і користувацький запит до вибраної моделі.
