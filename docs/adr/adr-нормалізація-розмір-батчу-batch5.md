# Обмеження розміру пакета ADR-нормалізації до 5

**Status:** Accepted
**Date:** 2026-05-18

## Context and Problem Statement

Скрипт `.claude/hooks/normalize-decisions.sh` передає пакети чернеток до `claude sonnet` через CLI. За замовчуванням пакет складав 30 файлів. Прогін із 30 файлами перевищив ліміт output-токенів (32k), а зменшення до 10 файлів призвело до `Request timed out` приблизно через годину виконання реального прогону.

## Considered Options

- `ADR_NORMALIZE_BATCH=30` (значення за замовчуванням)
- `ADR_NORMALIZE_BATCH=10` (перша пропозиція після помилки 32k токенів)
- `ADR_NORMALIZE_BATCH=5` (обрано після таймауту на BATCH=10)

## Decision Outcome

Chosen option: "`ADR_NORMALIZE_BATCH=5`", because пакет із 10 файлів призводив до `Request timed out` (~1 год), тоді як пакет із 5 стабільно завершується в межах лімітів output-токенів та часу виконання CLI.

### Consequences

- Good, because transcript фіксує очікувану користь: пакети по 5 файлів завершувались успішно (exit code 0) без перевищень часу очікування, операції merge-into / rewrite / delete застосовувались коректно.
- Bad, because для вичерпання великої черги чернеток потрібно запускати більше послідовних пакетів; для ~33 файлів знадобилося 4+ окремих прогонів.

## More Information

- Скрипт: `.claude/hooks/normalize-decisions.sh`
- Лог: `.claude/hooks/normalize-decisions.log`
- Команда ручного запуску: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=5 bash .claude/hooks/normalize-decisions.sh`
- Dry-run режим: `ADR_NORMALIZE_DRY=1` — виводить план без застосування змін на диск
- Модель: `claude sonnet` (через `claude -p sonnet`)
- Перша спроба (BATCH=30): dry-run завершився помилкою перевищення 32k output-токенів
- Друга спроба (BATCH=10): dry-run успішний; реальний прогін — `Request timed out` (~1 год)
- Прогони BATCH=5 (пакети 1–4+): успішні, exit code 0

## Update 2026-05-18

Додаткові деталі збоїв, зафіксовані в сесії `08f98514`:

- **BATCH=30** — dry-run впав через перевищення 32 k output-токенів (task ID `bn50mqhrp`).
- **BATCH=10** — один реальний прогон впав з `Request timed out` після ~1 год очікування (task ID `bm3ul8b0i`).
- **BATCH=5** — 9 послідовних пакетів завершились успішно; усі ~33 чернетки оброблені.

Команда ручного запуску:

```sh
ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=5 bash .claude/hooks/normalize-decisions.sh
```

Dry-run: додати `ADR_NORMALIZE_DRY=1`. Лог: `.claude/hooks/normalize-decisions.log`.

Повторний прогон підтвердив рішення: `BATCH=5` стабільно завершив 9 пакетів підряд (~33 чернетки) без overflow токенів та без timeout. Загальний час — ~7–10 хв на пакет × 9 пакетів.

Команда ручного запуску:

```
ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=5 bash .claude/hooks/normalize-decisions.sh
```

Dry-run: додати `ADR_NORMALIZE_DRY=1`. Лог: `.claude/hooks/normalize-decisions.log`. Скіл: `.cursor/skills/n-adr-normalize/SKILL.md`.

---

**Опрацьовано** 2026-05-20. Проекції:

- [decisions](../ci4/decisions.md)
