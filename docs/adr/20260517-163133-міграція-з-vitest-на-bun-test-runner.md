---
session: 18575de0-b5fa-4b72-a235-c55731b4c22a
captured: 2026-05-17T16:31:33+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/18575de0-b5fa-4b72-a235-c55731b4c22a.jsonl
---

## ADR Міграція з Vitest на Bun Test Runner
**Контекст:** У проєкті є два тест-файли (`auth-errors.test.js`, `auth-store.test.js`), які вже імпортують з `bun:test` і використовують `mock.module` — Bun-специфічний API, несумісний із Vitest. Скрипт `test` у `app/package.json` також уже викликає `bun test`. Vitest залишався у `devDependencies` без жодного реального тесту під його API.
**Рішення/Процедура/Факт:** Прийнято рішення офіційно закріпити **Bun Test Runner** як єдиний test runner у проєкті. Оновлено правило `.cursor/rules/n-vue.mdc` (версія 1.8 → 1.9): секція «Тестування» тепер рекомендує `bun:test` замість Vitest. Для DOM-тестів Vue-компонентів — `happy-dom` через `bunfig.toml` preload (`@happy-dom/global-registrator`). Vitest і `jsdom` підлягають видаленню з `devDependencies`.
**Обґрунтування:** Фактичний стан коду вже спирався на `bun:test`/`mock.module`, тобто шлях назад до Vitest фактично відрізаний. Bun Test Runner не потребує окремого конфіга, happy-dom швидший за jsdom і достатній для Vue-компонентного тестування. Уніфікація runnerа прибирає дублювання залежностей.
**Розглянуті альтернативи:** Гібридна схема — Vitest тільки для DOM-сюїти, Bun для юніт-тестів; відхилена як надмірно складна. `jsdom`-preload замість happy-dom — можливий запасний варіант при нестачі API у happy-dom.
**Зачіпає:** `app/package.json` (скрипти `test:*`, `devDependencies`), `.cursor/rules/n-vue.mdc`, потенційно новий `app/bunfig.toml` і `app/test/happy-dom.preload.js` при введенні DOM-тестів.
