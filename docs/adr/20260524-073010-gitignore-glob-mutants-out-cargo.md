# Glob `mutants.out*/` у `.gitignore` для покриття ротаційних директорій cargo-mutants

**Status:** Accepted
**Date:** 2026-05-24

## Context and Problem Statement

`cargo-mutants` при повторному запуску перейменовує попередній результат у `mutants.out.old/`. Існуючий `.gitignore` мав лише запис `mutants.out/`, тому `app/src-tauri/mutants.out.old/` залишався untracked і засмічував `git status`.

## Considered Options

- Замінити `mutants.out/` на glob `mutants.out*/`
- Додати окремий рядок `mutants.out.old/`

## Decision Outcome

Chosen option: "Замінити `mutants.out/` на glob `mutants.out*/`", because один glob покриває будь-яку кількість ротованих директорій (`mutants.out/`, `mutants.out.old/`, і потенційно майбутні варіанти) без дублювання рядків.

### Consequences

- Good, because `git check-ignore -v app/src-tauri/mutants.out.old/caught.txt` підтвердив спрацювання нового правила (`.gitignore:39:mutants.out*/`).
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінений файл: `.gitignore`, рядок 39: `mutants.out/` → `mutants.out*/`. Артефакти cargo-mutants у `app/src-tauri/mutants.out.old/`: caught.txt, debug.log, diff, lock.json, log, missed.txt, mutants.json, outcomes.json, timeout.txt, unviable.txt.

## Update 2026-05-24

Уточнення: коментар у `.gitignore` рядок 39 оновлено до `# cargo-mutants artifacts (mutants.out/ + mutants.out.old/ після повторних прогонів)`. Ts-nocheck-секція цього драфта покрита clean-ADR `20260524-070713-видалення-ts-nocheck-js-файли.md`.
