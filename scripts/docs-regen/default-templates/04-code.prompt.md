# Інструкції LLM для генерації `docs/ci4/04-code.md`

## Аудиторія

Інженер MLMaiL, який пише код.

## Призначення файлу

`docs/ci4/04-code.md` — Code (C4 рівень 4) застосунку MLMaiL. Конкретні файли, функції, конфігурація і операції.

## Обов'язкові секції

1. **Tauri-команди MLMaiL** — таблиця або bullet-list. Для кожної команди:
   - Назва (наприклад `gmail_inbox_count`).
   - Сигнатура Rust (повний return type).
   - Файл (`app/src-tauri/src/...`).
   - Короткий опис відповідальності.

2. **Vue-компоненти MLMaiL** ключових екранів — таблиця або bullet-list:
   - Назва (`Login.vue`, `Inbox.vue`).
   - Props.
   - Файл (`app/src/views/...` або `app/src/components/...`).
   - Які stores використовує.

3. **Конфігурація MLMaiL**:
   - `app/src-tauri/tauri.conf.json` — ключові секції.
   - Env vars (`app/src-tauri/.env`, `.env.example`).
   - OAuth client IDs, scopes.
   - Quasar variables (`app/src/quasar-variables.sass`).

4. **Operations MLMaiL** — Build, run, deploy:
   - Локальний dev (`bun run tauri dev`, `bun run android`).
   - Збірка macOS app bundle.
   - Збірка Android APK.
   - Lint, тести (`bun run lint`, `bun test`, `cargo test`).

## Інваріанти

- Усі шляхи — абсолютні відносно репо.
- Кожен запис самодостатній.

## Формат виводу

Дотримуйся `_global.prompt.md`.
