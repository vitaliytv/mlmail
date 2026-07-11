---
type: JS Module
title: main.js
resource: app/src/main.js
docgen:
  crc: 73e10ae0
---

## Огляд

Точка входу фронтенду: створює Vue-застосунок, підключає Quasar із набором
плагінів і монтує `App.vue` у DOM.

## Поведінка

- Підключає Quasar-плагіни `Dialog` і `Notify` — без `Dialog` виклики
  `$q.dialog(...)` (наприклад, у діалозі оновлення застосунку) падають з
  `TypeError: e.dialog is not a function`.
- Вмикає темну тему автоматично (`config.dark: 'auto'`, за системною
  темою ОС).
- Іконки — `material-symbols-outlined`.
