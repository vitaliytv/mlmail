# GitHub Actions matrix strategy у CI jobs

**Status:** Accepted
**Date:** 2026-05-22

## Context and Problem Statement

Правило `n-ga.mdc` вимагає застосовувати `strategy: matrix:` у всіх CI jobs. Команда `npx @nitra/cursor check` повертала `❌` на правилах `ga` і `ga (2)`, бо `.github/workflows/ci.yml` не містив ключового слова `strategy`.

## Considered Options

- Додати `strategy: matrix:` до jobs `backend` і `frontend`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Додати `strategy: matrix:` до jobs `backend` і `frontend`", because правило `n-ga.mdc` явно вимагає matrix strategy у всіх CI jobs.

### Consequences

- Good, because `npx @nitra/cursor check` проходить 12/12 — правила `ga` і `ga (2)` більше не порушені.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінений файл: `.github/workflows/ci.yml`.

- Job `backend`: `strategy.matrix.php-version: ["8.3"]`; `shivammathur/setup-php@v4` використовує `${{ matrix.php-version }}`
- Job `frontend`: `strategy.matrix.bun-version: ["latest"]`; `oven-sh/setup-bun@v2` використовує `${{ matrix.bun-version }}`

Виправлення застосовано у рамках запуску скілу `/n-fix` (сесія `e51cb74a`). Верифікація: повторний `npx @nitra/cursor check` повернув 12/12.
