---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T07:40:02+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

Зміни не застосував — чекаю підтвердження від тебе. Скажи, де в Infisical лежать секрети (є папка `/mlmail` чи стоять у корені env `main`)? Якщо у корені — дозволь правку, я поміняю `secret-path: '/mlmail'` → `secret-path: '/'`, закомічу і пушну.

---

## ADR: автоматичний реліз Tauri DMG+APK через GitHub Actions з Infisical OIDC

## Context and Problem Statement
Три Tauri-застосунки (mlmail, myshare, task) потребували автоматичного збирання та публікації бінарних артефактів (macOS DMG і Android APK) при кожному тегу `v*` у GitHub. Android-підпис потребує keystore та паролів, які небезпечно зберігати напряму в GitHub Secrets.

## Considered Options
* GitHub Actions Secrets (прямо зашифровані секрети в репо)
* Infisical GitHub Integration (push-синк із Infisical → GitHub Secrets)
* Infisical OIDC Machine Identity (CI отримує OIDC JWT від GitHub, Infisical верифікує і повертає секрети — секретів у GitHub взагалі немає)

## Decision Outcome
Chosen option: "Infisical OIDC Machine Identity", because це патерн, що вже використовується в `nitra`-проєктах (воркфлоу `gt-run.yml`), і він не потребує зберігання жодних секретів у GitHub — CI отримує OIDC-токен від GitHub, Infisical верифікує його й повертає `ANDROID_*` у env.

### Consequences
* Good, because Android-секрети (keystore base64 + паролі) лишаються виключно в Infisical; ротація відбувається в одному місці.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- `.github/workflows/release.yml` у `vitaliytv/mlmail`: `build-android` job, `permissions.id-token: write`, крок `Infisical/secrets-action@v1.0.16` з `method: 'oidc'`, `domain: 'https://secret.7n.ai'`, `project-slug: 'vitaliytv-kfse'`, `identity-id: '53691c96-17d9-4389-b078-0f77073809ab'`, `env-slug: 'main'`, `secret-path: '/mlmail'`.
- `app/src-tauri/gen/android/app/build.gradle.kts`: `signingConfigs { create("release") { ... } }` з fallback на debug якщо `keystore.properties` відсутній.
- Keystores згенеровані keytool (RSA 2048, 10000 днів) і збережені у `~/keystores-release/<app>/`.
- `task` (`nitra/task`) — desktop-only DMG поки що; Android не ініціалізований (`tauri android init` не виконувався).
