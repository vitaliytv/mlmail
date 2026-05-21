# `q-page` з `flex-center`: контент нижче viewport не видно

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

На сторінці `Login.vue` тіло листа (`body` з `gmail_random_message`) не відображалося в UI, попри те що Rust-команда повертала коректне значення і шаблон `{{ auth.currentMessage.value.body }}` присутній у DOM без жодного `v-if`.

## Considered Options

- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "замінити `flex flex-center` на `column items-center` на `q-page`", because клас `flex-center` встановлює `justify-content: center` і `align-items: center` одночасно; при контенті вищому за viewport вертикальне центрування зміщує його частково за нижній край без автоматичного scroll.

### Consequences

- Good, because контент починається від верху і природно виходить за нижній край — стандартна scroll-поведінка.
- Good, because горизонтальне центрування (`items-center`) зберігається.
- Neutral, because transcript не містить підтвердження про тестування рішення після впровадження.

## More Information

Дані (Rust → auth-store → шаблон) відстежені й підтверджені: `body` присутнє в DOM. Зникнення відтворюється лише при контенті, що перевищує висоту viewport.

Зачіпає: `app/src/views/Login.vue` (клас `q-page`).

---

**Опрацьовано** 2026-05-20. Проекції:

- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
