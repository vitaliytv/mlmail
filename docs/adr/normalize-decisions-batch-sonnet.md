# normalize-decisions.sh: обмеження розміру батчу до 10 для Claude Sonnet

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement
Хук `normalize-decisions.sh` при ручному запуску з 31 чернеткою та дефолтним батчем 30 завершився помилкою: відповідь Claude Sonnet перевищила ліміт у 32k output-токенів, і dry-run не завершився успішно.

## Considered Options
* `ADR_NORMALIZE_BATCH=30` (дефолтне значення) — зазнало збою через token limit
* `ADR_NORMALIZE_BATCH=10` — менший батч, що вписується в output-ліміт Sonnet

## Decision Outcome
Chosen option: "`ADR_NORMALIZE_BATCH=10`", because при 30 файлах у батчі Claude Sonnet перевищував 32k output-токенів; зменшення до 10 дозволяє обробляти кожен батч у межах ліміту моделі.

### Consequences
* Good, because батч із 10 чернеток не перевищує output-ліміт Sonnet — dry-run завершується успішно та формує повний план операцій.
* Bad, because при великому пулі чернеток потрібно кілька послідовних прогонів: 33 чернетки потребують ≥4 батчів.

## More Information
- Dry-run: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=10 ADR_NORMALIZE_DRY=1 bash .claude/hooks/normalize-decisions.sh`
- Реальний прогін: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=10 bash .claude/hooks/normalize-decisions.sh`
- Скрипт: `.claude/hooks/normalize-decisions.sh`
- Лог: `.claude/hooks/normalize-decisions.log`

## Update 2026-05-17

Підтвердження з окремого dry-run того ж батчу в сесії 08f98514:
- Перший dry-run (batch=30): task ID `bn50mqhrp`, завершився помилкою через перевищення 32k output-токенів
- Успішний dry-run (batch=10): task ID `buv4xfiq8`, exit code 0, сформовано план із 10 операцій (2× merge-into, 4× delete, 4× rewrite)
- Змінна `ADR_NORMALIZE_BATCH` перевизначає дефолт без зміни коду скрипту

## Update 2026-05-17

Додаткова деталь із transcript: dry-run з BATCH=10 завершився успішно (~10 хв), але реальний прогін з BATCH=10 завис приблизно на 1 годину й завершився помилкою `Request timed out`. Остаточний вибір — **BATCH=5**, при якому прогін завершився з exit code 0.

Guardrail від рекурсії: паралельні Stop-хук-виклики під час виконання логували `skip: only N s since last attempt` — основний процес нормалізації при цьому не переривався.

## Update 2026-05-18

Повна хронологія підбору розміру батчу:
- **BATCH=30** (дефолт): dry-run завершився помилкою `exceeds 32k output tokens`
- **BATCH=10**: dry-run успішний (~10 хв); реальний прогін зависнув ~1 год, завершився `Request timed out`
- **BATCH=5**: реальний прогін завершився успішно (exit code 0) — обраний як робочий розмір

Деталі перших двох батчів при BATCH=5:
- Батч 1: 2× merge-into `ADR-0006-google-oauth.md`, 2× rewrite (`oauth-client-ids-runtime-dotenvy.md`, `oauth-client-id-у-приватному-репо.md`), 1× delete
- Батч 2: 5 операцій (деталі в лозі `2026-05-17T22:10:36+03:00`)
- Після 2 батчів лишалось 29 чернеток; батч 3 запущено у фоні

Команда запуску: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=5 bash .claude/hooks/normalize-decisions.sh`

Лог операцій: `.claude/hooks/normalize-decisions.log`

---

**Опрацьовано** 2026-05-19. Проекції:
- [decisions](../ci4/decisions.md)
