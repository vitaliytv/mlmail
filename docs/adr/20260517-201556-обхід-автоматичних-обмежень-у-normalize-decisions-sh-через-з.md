---
session: 08f98514-3887-4e6a-abe5-0d92e0029504
captured: 2026-05-17T20:15:56+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/08f98514-3887-4e6a-abe5-0d92e0029504.jsonl
---

## ADR Обхід автоматичних обмежень у `normalize-decisions.sh` через змінні середовища

## Context and Problem Statement
Скрипт `.claude/hooks/normalize-decisions.sh` має вбудований time-gate (`min_interval = 21600 s`) і threshold на кількість чернеток — автоматичні запуски пропускаються, якщо умови не виконані. Під час ручного запуску (скіл `/n-adr-normalize`) ці обмеження заважають обробці, бо оператор свідомо хоче примусово нормалізувати батч.

## Considered Options
* Передати `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0` як env-змінні для обходу обох guard-ів
* `ADR_NORMALIZE_DRY=1` — dry-run режим для попереднього перегляду без змін
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "env-var override при ручному запуску", because скіл `.cursor/skills/n-adr-normalize/SKILL.md` явно запускає скрипт з `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_DRY=1`, що підтверджено Bash-командою в transcript.

### Consequences
* Good, because transcript фіксує очікувану користь: автоматичні `skip:`-рядки в `.claude/hooks/normalize-decisions.log` більше не блокують ручний прогін.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Команда запуску: `ADR_NORMALIZE_THRESHOLD=0 ADR_NORMALIZE_MIN_INTERVAL_HOURS=0 ADR_NORMALIZE_DRY=1 bash .claude/hooks/normalize-decisions.sh`
- Лог-файл: `.claude/hooks/normalize-decisions.log` — рядки вигляду `skip: only N s since last attempt (min 21600s)` підтверджують роботу time-gate при звичайних хук-викликах
- У transcript зафіксовано recursion guard: паралельні виклики від Stop-події блокуються, поки основний процес (PID 68884) активний
- Скрипт обробляє батч із 30 чернеток через `claude sonnet`; знайдено 31 драфт
