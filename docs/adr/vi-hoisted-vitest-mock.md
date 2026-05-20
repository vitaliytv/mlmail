# vi.hoisted() обов'язковий для mock-об'єктів з vi.fn() у Vitest

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

Під час написання тестів для `Login.vue` виникла помилка `Cannot use vi.fn() outside of a mock factory` при оголошенні спільного mock-об'єкта з `vi.fn()` як звичайної `const` поруч з `vi.mock()`.

## Рішення/Процедура/Факт

Vitest hoistує `vi.mock()` на початок файлу на етапі трансформації — до будь-якого іншого коду. `vi.fn()` у звичайній `const` викликається до ініціалізації Vitest runtime, звідси помилка. Рішення — `vi.hoisted(() => ({...}))`: цей API виконується у тій самій фазі hoisting, що й `vi.mock`, тому змінна доступна у фабриці та у тілі тестів:

```js
const mockAuth = vi.hoisted(() => ({
  status: 'idle',
  initialize: vi.fn(),
  startLogin: vi.fn()
}))
vi.mock('../services/auth-store.js', () => ({ useAuthStore: () => mockAuth }))
```

## Обґрунтування

`vi.hoisted` гарантує виклик `vi.fn()` після ініціалізації runtime, а змінна залишається доступною у фабриці та тестах для перевірки викликів.

## Розглянуті альтернативи

Перемістити `vi.fn()` всередину фабрики `vi.mock` — тоді немає доступу до них у тестах для assertions.

## Зачіпає

`app/src/views/Login.test.js`, всі майбутні Vitest-тести зі спільними mock-об'єктами.

---

**Опрацьовано** 2026-05-20. Проекції:
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
