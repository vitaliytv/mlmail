---
session: ae4f836e-0b41-4f34-a3d2-00b89102ef22
captured: 2026-05-24T07:44:27+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/ae4f836e-0b41-4f34-a3d2-00b89102ef22.jsonl
---

## ADR Видалення `// @ts-nocheck` замість глушіння помилок TypeScript

## Context and Problem Statement

Попередній агент додав `// @ts-nocheck` на перший рядок 9 файлів проєкту (`app/src/App.test.js`, `app/src/services/auth-store.test.js`, `app/src/views/Login.test.js`, `app/src/i18n/auth-errors.test.js`, `app/src/test-utils/quasar.js`, `app/src/vite-env.d.ts`, `app/vite.config.js`, `app/test/happy-dom.preload.js`, `app/stryker.config.mjs`). Жодне правило у `.cursor/rules/` цього не вимагало; зміни були uncommitted (`git blame app/src/App.test.js:1` — `Not Committed Yet 2026-05-23`).

## Considered Options

- Залишити `// @ts-nocheck` і жити зі «сліпими» зонами для type-checker
- Видалити `// @ts-nocheck` і верифікувати через `bunx tsc -p jsconfig.json --noEmit`

## Decision Outcome

Chosen option: "Видалити `// @ts-nocheck` і верифікувати через `bunx tsc`", because `bunx tsc -p app/jsconfig.json --noEmit` повернув exit 0, 0 помилок (TypeScript 6.0.3) — коментарі були cargo-культом без реальної потреби. `app/auto-imports.d.ts` свідомо виключений: він автогенерується `unplugin-auto-import` і ставить `@ts-nocheck` сам.

### Consequences

- Good, because transcript фіксує очікувану користь: type-checker тепер покриває всі 9 файлів без «сліпих» зон; `app/src/vite-env.d.ts` відновлений до канонічного вигляду (`/// <reference types="vite/client" />`), як вимагає правило `n-vue.mdc`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

`app/jsconfig.json` — `"strict": true`, `"allowJs": true`, `"include": ["src/**/*"]`; файли поза `src/` (`vite.config.js`, `test/happy-dom.preload.js`, `stryker.config.mjs`) взагалі не входять до scope `jsconfig.json`, тому для них `@ts-nocheck` не мав жодного ефекту. Окремий перегін `bunx tsc --noEmit --allowJs --strict … vite.config.js test/happy-dom.preload.js stryker.config.mjs` також повернув exit 0.

---

## ADR Glob `mutants.out*/` у `.gitignore` для покриття ротаційних директорій cargo-mutants

## Context and Problem Statement

`.gitignore` містив pattern `mutants.out/`, але cargo-mutants при повторному запуску перейменовує попередній результат у `mutants.out.old/`. Вміст `app/src-tauri/mutants.out.old/` (`caught.txt`, `debug.log`, `diff/`, `lock.json`, `log/`, `missed.txt`, `mutants.json`, `outcomes.json`, `timeout.txt`, `unviable.txt`) показувався у `git status` як untracked.

## Considered Options

- Розширити glob до `mutants.out*/` (покриває будь-який суфікс)
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Розширити glob до `mutants.out*/`", because один glob закриває і `mutants.out/`, і `mutants.out.old/`, і будь-які майбутні варіанти ротації; `git check-ignore -v app/src-tauri/mutants.out.old/caught.txt` підтвердив спрацювання нового правила.

### Consequences

- Good, because transcript фіксує очікувану користь: `git status` більше не показує артефакти cargo-mutants як untracked.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінений файл: `.gitignore`, рядок 39. До: `mutants.out/`. Після: `mutants.out*/`. Коментар оновлено до: `# cargo-mutants artifacts (mutants.out/ + mutants.out.old/ після повторних прогонів)`.
