---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T10:07:02+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

The transcript ends here.

---

## ADR Автоматичний реліз Tauri-застосунку через GitHub Actions з Infisical OIDC

## Context and Problem Statement
Проєкти `mlmail` і `myshare` (Tauri 2, bun-монорепо) потребували механізму автоматичного релізу, що публікує DMG (macOS) і APK (Android) на GitHub Releases при push git-тегу. Управління Android-секретами (keystore, паролі) мало відповідати вже наявній у репозиторіях Infisical-практиці без зберігання секретів у GitHub Actions Secrets.

## Considered Options
* GitHub Actions Secrets (ANDROID_KEYSTORE_BASE64 тощо покласти напряму в репо)
* Infisical Machine Identity з Universal Auth (client-id + client-secret у GH Secrets)
* Infisical Machine Identity з OIDC Auth (`method: 'oidc'`, `id-token: write` — нуль секретів у GitHub)

## Decision Outcome
Chosen option: "Infisical Machine Identity з OIDC Auth", because цей патерн вже використовується в репо `nitra/ai` (Infisical/secrets-action@v1.0.16, method=oidc, identity `d43949cb`) і не потребує жодних секретів у GitHub — GitHub видає OIDC JWT, Infisical його верифікує за Subject-паттерном і повертає секрети у змінні середовища.

### Consequences
* Good, because Android-секрети (`ANDROID_KEYSTORE_BASE64`, паролі) зберігаються лише в Infisical (проєкт `vitaliytv-kfse`, env `main`, шлях `/mlmail`), а не в GitHub Actions Secrets.
* Good, because transcript фіксує очікувану користь: `build-desktop` успішно зібрав і завантажив `mlmail_0.1.0_universal.dmg` після першого тегу `v0.1.1`.
* Bad, because налаштування вимагає трьох дій у дашборді Infisical, які CLI не підтримує: (1) OIDC Auth на identity, (2) Subject-патерн що покриває і гілки і теги (`repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}`), (3) membership identity у конкретному проєкті. Два помилкових Subject-паттерни призвели до кількох retry-циклів у transcript.

## More Information
- `.github/workflows/release.yml` — тригер `on.push.tags: ['v*']`; три джоби: `create-release`, `build-desktop` (macOS, `tauri-apps/tauri-action@v0`), `build-android` (Ubuntu, `Infisical/secrets-action@v1.0.16` → `bun --cwd=app run tauri android build --apk`)
- `app/src-tauri/gen/android/app/build.gradle.kts` — додано `signingConfigs.release` з fallback на debug-підпис якщо `keystore.properties` не існує
- Infisical: проєкт `vitaliytv-kfse`, identity `53691c96-17d9-4389-b078-0f77073809ab`, env `main`, secret-path `/mlmail`
- Keystore для `mlmail` і `myshare` згенеровано локально (`~/keystores-release/<app>/`), поза git-репозиторіями
- Баг `bun --cwd app` (пробіл замість `=`) призводить до того що bun друкує власний usage і виходить без запуску tauri; виправлено на `bun --cwd=app` у commit `c6fd100`
- `Sync app version from tag` — крок у обидві джоби, що переписує `version` у `app/src-tauri/tauri.conf.json` зі значення тегу перед збіркою; забезпечує що ім'я DMG/APK збігається з тегом релізу
