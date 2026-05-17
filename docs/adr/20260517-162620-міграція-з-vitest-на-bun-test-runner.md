---
session: 18575de0-b5fa-4b72-a235-c55731b4c22a
captured: 2026-05-17T16:26:20+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/18575de0-b5fa-4b72-a235-c55731b4c22a.jsonl
---

[thinking]

[assistant]
Ні, у локфайлі `happy-dom` з'являється як **транзитивна** залежність `vitest` — не як самостійна. У `app/package.json` він не підключений.

---

**Так, з Bun Test Runner + happy-dom DOM-тести працюють**, але потрібна мінімальна конфігурація:

1. Додати в `app/package.json` devDependencies:

```json
"happy-dom": "^16.0.0"
```

2. Створити `app/bunfig.toml` (або додати до існуючого):

```toml
[test]
preload = ["./setup-happy-dom.ts"]
```

3. Файл `app/setup-happy-dom.ts`:

```ts
import { GlobalRegistrator } from '@happy-dom/global-registrator'
GlobalRegistrator.register()
```

Після цього `document`, `window`, `HTMLElement` доступні у тестах — `@vue/test-utils` теж підхопить.

---

**Висновок:** Vitest можна прибирати повністю. DOM-тести через `happy-dom` + Bun Test Runner підтримуються без Vitest. Виконати видалення Vitest і налаштування happy-dom?
[user]
Не потрібно, просто хотів зрозуміти чи можливо
[assistant]
[thinking]

[assistant]
Зрозуміло. Так, можливо — і у вас уже майже все готово для повного переходу.

[user]
[assistant]

---

Based on the session, I'll capture this as an ADR since it covers a design choice about the test runner migration.

## ADR Міграція з Vitest на Bun Test Runner

**Контекст:** У проєкті `app/` паралельно існували залежності Vitest і скрипт `bun test`, що створювало дублювання. Виникло питання: чи можна повністю перейти на Bun Test Runner.

**Рішення/Процедура/Факт:** Рішення — перейти на Bun Test Runner як єдиний test runner. Обидва наявні тести (`auth-errors.test.js`, `auth-store.test.js`) уже імпортують із `bun:test` та використовують `mock.module()` (Bun API), а скрипт `test` в `app/package.json` запускає `bun test`. `vitest.config.*` відсутній. Для DOM-тестів — підключити `happy-dom` через `bunfig.toml` + `GlobalRegistrator.register()` замість `jsdom`. Повний перехід: видалити `vitest`, `jsdom` з devDependencies; замінити скрипти `test:watch` / `test:ui` / `test:all` на Bun-варіанти.

**Обґрунтування:** Обидва тест-файли вже прив'язані до `bun:test` API (зокрема `mock.module`) — повернення на Vitest технічно неможливе без рефакторингу. `happy-dom` покриває DOM-потреби без Vitest.

**Розглянуті альтернативи:** Гібридна схема — Vitest тільки для DOM-тестів, Bun для юніт-тестів — відхилена як ускладнення конфігурації.

**Зачіпає:** `app/package.json`, `app/src/i18n/auth-errors.test.js`, `app/src/services/auth-store.test.js`, майбутній `app/bunfig.toml`
