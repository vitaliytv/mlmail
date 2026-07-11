---
type: JS Module
title: newsletter-template.js
resource: app/src/services/newsletter-template.js
docgen:
  crc: 618c12d8
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.99
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Файл надає публічні функції `listTemplates`, `saveTemplate`, `deleteTemplate`, `saveBuiltinTemplate`, `findTemplateForMessage`, `slugify`, `SYSTEM_PROMPT` для роботи з newsletter-шаблонами та пов’язаною логікою вибору шаблону для повідомлення. Також тут зберігається системна інструкція для обробки newsletter-контенту, яка орієнтована на повернення лише реальних URL із листа, зокрема `https://...`. Через звернення до мережі цей модуль підтримує актуальність даних під час роботи з шаблонами та вмістом листів.

## Поведінка

- `listTemplates` — повертає список шаблонів newsletter із застосунку.
- `saveTemplate` — зберігає шаблон newsletter у застосунку.
- `deleteTemplate` — видаляє шаблон newsletter за його `id`.
- `saveBuiltinTemplate` — зберігає вбудований системний шаблон для dev-режиму.
- `findTemplateForMessage` — знаходить перший шаблон, що підходить до листа за відправником і/або темою, і повертає його або `null`.
- `slugify` — перетворює рядок на slug-ідентифікатор для домену або email.
- `SYSTEM_PROMPT` — задає інструкцію для витягування статей із newsletter у JSON, з вимогою повертати лише реальні URL з листа, зокрема посилання на кшталт `https://...`.

## Публічний API

Почну з читання потрібних правил і цільового файлу, щоб написати документацію у вашому форматі.

## Гарантії поведінки

- (специфічних машинно-виведених гарантій немає)
