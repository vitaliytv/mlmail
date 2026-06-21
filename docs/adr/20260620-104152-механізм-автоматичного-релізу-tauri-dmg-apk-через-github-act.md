---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T10:41:52+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Механізм автоматичного релізу Tauri (DMG + APK) через GitHub Actions і Infisical OIDC

## Context and Problem Statement
Три Tauri-монорепо (`mlmail`, `myshare`, `nitra/task`) потребують автоматизованого релізного пайплайну: push git-тегу `v*` має збирати universal DMG (macOS) і підписаний APK (Android) та завантажувати їх у GitHub Release. Секрети для підпису APK необхідно зберігати поза GitHub Secrets за стандартом організації.

## Considered Options
* Push тегу `v*` як тригер + `tauri-apps/tauri-action` для DMG + `tauri android build --apk` для APK
* Інші варіанти тригера або збірника в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Push тегу `v*` як тригер; три джоби (`create-release`, `build-desktop`, `build-android`); секрети через Infisical OIDC (`Infisical/secrets-action@v1.0.16`, `method: oidc`)", because це відповідає наявному `nitra`-патерну (жодних секретів у GitHub, лише `id-token: write`), і користувач прямо вказав на `Варіант А — синк з Infisical` як бажану схему, підтвердивши OIDC-шлях у ході налаштування identity.

### Consequences
* Good, because GitHub Release отримує `mlmail_0.1.2_universal.dmg` автоматично після `git push origin v0.1.2`; версія артефакту синхронізується з тегом через крок «Sync app version from tag» у `tauri.conf.json`.
* Bad, because transcript фіксує три итерації налагодження OIDC: (1) OIDC Subject не містив `refs/tags/*` → додано `repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}`; (2) identity не була членом проєкту `vitaliytv-kfse` → додана вручну в дашборді; (3) PKCS12-keystore ігнорує окремий `-keypass`, тому `ANDROID_KEY_PASSWORD` має дорівнювати `ANDROID_KEYSTORE_PASSWORD` — без цього `PackageAndroidArtifact` падає з `Get Key failed: Given final block not properly padded`.

## More Information
- Файли змінено: `.github/workflows/release.yml` (mlmail, myshare), `.github/workflows/clean-merged-branch.yml` (додано `pull-requests: read`), `app/src-tauri/gen/android/app/build.gradle.kts` (mlmail, myshare) — `signingConfigs { create("release") }` з fallback на debug.
- Infisical: домен `https://secret.7n.ai`, project-slug `vitaliytv-kfse`, identity-id `53691c96-17d9-4389-b078-0f77073809ab`, env `main`, secret-path `/mlmail`.
- Keystores: `~/keystores-release/<app>/<app>-release.jks` (PKCS12, alias = назва застосунку). Для PKCS12 `keyPassword = storePassword`.
- Діагностована помилка `bun --cwd app` (пробіл) → виправлено на `bun --cwd=app` у всіх трьох `release.yml`.
- `nitra/task` — desktop-only (APK відсутній до `tauri android init`).
- Commits: `6db7343`, `0e34468`, `c6fd100` у `vitaliytv/mlmail@main`.
