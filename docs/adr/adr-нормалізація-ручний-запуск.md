# ADR-нормалізація: ручний запуск через скіл n-adr-normalize

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

Скрипт `normalize-decisions.sh` нормалізує ADR-чернетки автоматично після кожної сесії, але лише якщо виконуються умови: кількість чернеток ≥ порогу (`THRESHOLD=3`) і минув мінімальний інтервал (`MIN_INTERVAL_HOURS=4`). Коли порогові умови не виконуються, потрібен спосіб запустити нормалізацію вручну в обхід цих обмежень.

## Considered Options

- Ручний запуск `normalize-decisions.sh` з `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Ручний запуск з env-var override", because скіл `n-adr-normalize` визначає саме цей підхід для обходу порогу й min-interval без зміни коду скрипту.

### Consequences

- Good, because оператор може нормалізувати чернетки будь-коли, не чекаючи автоматичного тригера.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- Dry-run: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_DRY=1 bash .claude/hooks/normalize-decisions.sh`
- Реальний запис: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 bash .claude/hooks/normalize-decisions.sh`
- Файл стану: `.claude/hooks/.normalize-state`
- Скіл: `.cursor/skills/n-adr-normalize/SKILL.md`

## Update 2026-05-17

Додаткові деталі, зафіксовані в тій самій сесії (08f98514):

- Recursion guard: паралельні виклики від Stop-події блокуються, поки основний процес активний (PID зафіксовано в transcript)
- Time-gate підтверджений: `.claude/hooks/normalize-decisions.log` містить рядки `skip: only N s since last attempt (min 21600s)` при автоматичних хук-викликах
- При 31 чернетці в пулі дефолтний пакет (30) перевищив output-token ліміт Claude Sonnet — потребує зменшення пакета (див. `normalize-decisions-batch-sonnet.md`)

## Update 2026-05-18

**Dry-run перед реальним прогоном**: `normalize-decisions.sh` виконує незворотні файлові операції (delete, merge-into, rewrite). Перед застосуванням рекомендується запускати скрипт із `ADR_NORMALIZE_DRY=1` — план LLM відображається в `.claude/hooks/normalize-decisions.log` і дає змогу виявити некоректні операції до запису файлів.

- Команда dry-run: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_BATCH=5 ADR_NORMALIZE_DRY=1 bash .claude/hooks/normalize-decisions.sh`
- Команда реального прогону: те саме без `ADR_NORMALIZE_DRY=1`
- Приклад результату dry-run (BATCH=10): 10 операцій — 2× merge-into, 4× delete, 4× rewrite — переглянуто перед запуском.

Dry-run (`ADR_NORMALIZE_DRY=1`) підтверджено як обов'язковий крок перед реальним прогоном нормалізації. Скіл `n-adr-normalize` закріплює цей порядок; в transcript зафіксовано, що план після dry-run переглядався (2× merge-into, 4× delete/rewrite) перед підтвердженням реального запуску.

- Dry-run команда: `ADR_NORMALIZE_DRY=1 bash .claude/hooks/normalize-decisions.sh`
- Змінна `ADR_NORMALIZE_DRY=1` активує режим без запису на диск
- Додатковий час виконання: кожен пакет проходить двічі (dry + real)

---

**Опрацьовано** 2026-05-20. Проекції:

- [decisions](../ci4/decisions.md)
