---
type: JS Module
title: quasar.js
resource: app/src/test-utils/quasar.js
docgen:
  crc: 513cd43f
  model: openai-codex/gpt-5.4
  tier: cloud-avg
  score: 100
  issues: judge:inaccurate:0.98
  judgeModel: openai-codex/gpt-5.4-mini
---

## Огляд

Файл зберігає сумісність для імпорту зі старого шляху `app/src/test-utils/quasar.js`. Він повторно експортує `mountQuasar` і `mountWithQuasar` з пакета `@7n/tauri-components/testing`, щоб тести, які звертаються до цього шляху, і далі отримували ті самі точки входу без зміни імпорту, наприклад: `import { mountQuasar, mountWithQuasar } from 'app/src/test-utils/quasar.js'`.

## Поведінка

1. Зберігає сумісність існуючих тестів, які звертаються до локального модуля `app/src/test-utils/quasar.js`, щоб їх не потрібно було масово переписувати після винесення helpers у спільний пакет.
2. Перенаправляє використання `mountQuasar` і `mountWithQuasar` на спільну реалізацію з `@7n/tauri-components/testing`, щоб усі тести спиралися на єдину поведінку mounting для Quasar.
3. Не додає власної логіки, стану чи побічних ефектів, тому поводиться як прозорий шар сумісності між старим шляхом імпорту та новим джерелом helpers.
4. Працює в режимі read-only щодо локального модуля: сам файл нічого не зберігає і не змінює у файловій системі чи базі даних.

## Гарантії поведінки

- Read-only: не виконує операцій запису (ФС/БД).
