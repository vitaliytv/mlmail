# Tauri — відсутність web-цілі та архітектурні наслідки

**Status:** Accepted
**Date:** 2026-05-14

## Context and Problem Statement

Команда використовує Tauri для desktop+mobile у проєкті MLMaiL і потребувала пояснення, чи можна ту саму Tauri-збірку розгорнути у браузер, як це дозволяє Capacitor.

## Considered Options

- Окреме web-розгортання через Vite як PWA з шаром абстракції `platform.ts`
- Компіляція Rust-логіки у WASM через `wasm-bindgen` для web (парсери, crypto, векторні представлення)
- Capacitor `@capacitor/web` з no-op заглушками — одна збірка для WebView і браузера
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Окреме web-розгортання із шаром `platform.ts`", because Tauri не має `web`-платформи і frontend розгортається окремо через Vite; тонкий шар `platform.ts` абстрагує IPC (Tauri: `invoke`, web: `fetch`/WASM).

### Consequences

- Good, because Rust-бізнес-логіку (парсери, crypto, векторні представлення) можна компілювати у WASM через `wasm-bindgen` для web.
- Good, because `candle` і `ort` мають WASM-збірки — inference у браузері без GPU можливий.
- Bad, because FS, keychain і OS API недоступні у WASM — там потрібен окремий бекенд.
- Bad, because для важких моделей inference у браузері потребує серверного бекенду.
- Neutral, because на Android Rust компілюється до NDK через JNI напряму, без Java/Kotlin у бізнес-логіці.

## More Information

**Шар абстракції:** `app/src/platform.ts` — на Tauri: `invoke('cmd', args)`, на web: `fetch('/api/cmd')` або WASM-виклик.

**Потенційні файли:** `src-tauri/src/` (Rust-команди), `wasm/` (пакет для web-цілі).

Додаткової інформації в transcript не зафіксовано щодо конкретного плану реалізації web-розгортання для MLMaiL.

---

**Опрацьовано** 2026-05-20. Проекції:

- [01-context](../ci4/01-context.md)
- [decisions](../ci4/decisions.md)
