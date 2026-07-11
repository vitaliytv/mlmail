---
type: JS Module
title: pattern.js
resource: app/src/services/pattern.js
docgen:
  crc: d5433895
  model: openai-codex/gpt-5.4-mini
  tier: cloud-min
  score: 100
  issues: judge:inaccurate:0.97
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Файл надає три поведінкові перетворення для поштових рядків: `parseFromEmail` витягує email-адресу з тексту, `buildPatternQuery` формує пошуковий запит для `sender` і `subject`, а `sanitizeSubjectSuggestion` приводить чернетку subject-підказки до придатного значення без зайвих символів оформлення. Це потрібно, щоб далі в системі можна було надійно використовувати дані з листа у пошуку та підстановках без ручного очищення.

## Поведінка

- `parseFromEmail` — витягує чисту email-адресу з поля From; якщо адреси в кутових дужках немає, повертає trimmed-вхід.
- `buildPatternQuery` — збирає Gmail search query з sender і/або subject, прибираючи зайві лапки з subject, щоб запит лишався коректним.
- `sanitizeSubjectSuggestion` — приводить чернетку subject-підказки до придатного значення: бере перший рядок, знімає обрамлювальні лапки/бектики та використовує fallback, якщо результат порожній або невалідний.

## Публічний API

- parseFromEmail — витягує чисту email-адресу з сирого `From`-заголовка; якщо адреса вже без обгортки, повертає її без змін.
- buildPatternQuery — збирає Gmail search-запит із sender і/або subject-фрази; прибирає вкладені лапки, щоб `subject:"…"` лишався синтаксично цілісним.
- sanitizeSubjectSuggestion — бере перший рядок підказки від LLM, знімає обрамлення з лапок і backticks та підставляє `fallback`, якщо результат порожній або не є рядком.

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
