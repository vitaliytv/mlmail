---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T10:40:30+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Infisical OIDC замість GitHub Secrets для Android-підпису

## Context and Problem Statement
Три Tauri-застосунки (mlmail, myshare, task) потребують CI-доступу до release keystore (4 секрети `ANDROID_*`) для підпису APK. Потрібно вибрати спосіб доставки цих секретів у GitHub Actions без зберігання їх безпосередньо в GitHub Secrets.

## Considered Options
* GitHub Actions Secrets (поклавши 4 `ANDROID_*` безпосередньо в репо)
* Infisical GitHub-інтеграція (push-синк секретів в Actions Secrets)
* Infisical Machine Identity OIDC (`Infisical/secrets-action` + `method: oidc`)

## Decision Outcome
Chosen option: "Infisical Machine Identity OIDC", because цей патерн уже використовується в org nitra (репо `nitra/ai`), він не вимагає зберігати Android-секрети в GitHub взагалі — GA отримує OIDC JWT від GitHub, Infisical його валідує й інжектить `ANDROID_*` у середовище білду в runtime.

### Consequences
* Good, because transcript фіксує очікувану користь: Android-секрети зберігаються лише в Infisical (проєкт `vitaliytv-kfse`, env `main`, path `/mlmail`); GitHub Secrets для репо лишаються порожніми, що підтверджено `gh secret list -R vitaliytv/mlmail`.
* Bad, because потребує ручного налаштування в дашборді Infisical: OIDC Auth на identity (`53691c96-17d9-4389-b078-0f77073809ab`) з правильним Subject-патерном (`repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}`) і членства identity у проєкті (Project → Access Control → Identities). CLI `infisical` ці кроки не виконує.

## More Information
- `Infisical/secrets-action@v1.0.16`, параметри: `method: oidc`, `domain: https://secret.7n.ai`, `project-slug: vitaliytv-kfse`, `identity-id: 53691c96-17d9-4389-b078-0f77073809ab`, `env-slug: main`.
- Перший Subject-патерн (`repo:{nitra,vitaliytv}/*:ref:refs/heads/{dev,main}`) не охоплював теги → `403 OIDC subject not allowed`. Фікс: додано `tags/*` у вкладені дужки.
- Другий 403 (`ProjectMembershipNotFound`) усунено додаванням identity до проєкту в Infisical-дашборді.
- Файл: `.github/workflows/release.yml` (jobs: `build-android`), коміти `6db7343`, `0e34468`, `c6fd100` у `vitaliytv/mlmail`.

---

## ADR Видалення rust-cache з release-workflow

## Context and Problem Statement
Під час `actionlint`/`zizmor` перевірки nового `.github/workflows/release.yml` для mlmail зімор підняв попередження рівня High на крок `Swatinem/rust-cache@v2` у release-контексті через ризик cache-poisoning.

## Considered Options
* Залишити `rust-cache` (прийняти ризик зімора, збірки швидші)
* Видалити `rust-cache` з release-workflow (релізи рідкісні — кеш не критичний)

## Decision Outcome
Chosen option: "Видалити rust-cache з release-workflow", because релізи запускаються рідко (на тег), тому час збірки менш критичний, а уникнення cache-poisoning у release-контексті важливіше. Зімор після видалення повернув `No findings to report`.

### Consequences
* Good, because transcript фіксує очікувану користь: `zizmor: No findings to report. Good job! (31 suppressed)` після видалення.
* Bad, because час Rust-збірки у release-jobs довший (~19 хв для Android, ~5 хв для macOS) без кешування.

## More Information
- Видалено кроки `Swatinem/rust-cache@v2` з jobs `build-desktop` і `build-android` у `.github/workflows/release.yml`.
- Upload артефактів переведено на `gh release upload` замість `softprops/action-gh-release`, що також усунуло зайвий сторонній екшен.

---

## ADR Автосинк версії tauri.conf.json із git-тегу

## Context and Problem Statement
`tauri-action` і Gradle беруть версію застосунку з `app/src-tauri/tauri.conf.json`, тому артефакти релізу v0.1.1 мали назву `mlmail_0.1.0_universal.dmg` (версія зі статичного конфігу, а не з тегу), що викликало розбіжність між тегом і версією артефакту.

## Considered Options
* Вручну оновлювати `version` у `tauri.conf.json` перед кожним тегом
* Автоматично переписувати `version` у `tauri.conf.json` зі значення тегу кроком у CI

## Decision Outcome
Chosen option: "Автоматично переписувати version у tauri.conf.json зі значення тегу кроком у CI", because тег стає єдиним джерелом правди — версія застосунку та назви артефактів завжди збігаються з `github.ref_name` без ручних змін.

### Consequences
* Good, because transcript фіксує очікувану користь: після застосування кроку в релізі v0.1.2 DMG отримав назву `mlmail_0.1.2_universal.dmg`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Крок «Sync app version from tag» доданий в обидва jobs (`build-desktop`, `build-android`) у `.github/workflows/release.yml`:
```bash
VER="${GITHUB_REF_NAME#v}"
node -e "const f='app/src-tauri/tauri.conf.json'; const c=require('fs').readFileSync(f,'utf8'); require('fs').writeFileSync(f,c.replace(/\"version\":\s*\"[^\"]+\"/,\`\"version\":\"${VER}\"\`))"
```
- Коміт `c6fd100` у `vitaliytv/mlmail`.

---

## ADR PKCS12 keystore ігнорує окремий keyPassword

## Context and Problem Statement
Під час генерації release keystore через `keytool` були передані окремі значення `-storepass` та `-keypass`. У Infisical залиті відповідно відмінні `ANDROID_KEYSTORE_PASSWORD` і `ANDROID_KEY_PASSWORD`. Gradle `signingConfig` падав з `KeytoolException: Failed to read key *** from store: Given final block not properly padded`.

## Considered Options
* Залишити різні значення storePassword і keyPassword (як у JKS-форматі)
* Використовувати однакове значення (storePassword) для обох, оскільки PKCS12 ігнорує окремий keyPassword

## Decision Outcome
Chosen option: "Використовувати однакове значення для обох", because `keytool` при генерації PKCS12 keystore ігнорує переданий `-keypass` (підтверджено локально: `keytool -certreq` приймає будь-який keypass, навіть `WRONG_PASSWORD_123`). Реальний ключ завжди захищений storePassword.

### Consequences
* Good, because transcript фіксує очікувану користь: локальні файли `~/keystores-release/mlmail/secrets/ANDROID_KEY_PASSWORD.txt` і `~/keystores-release/myshare/secrets/ANDROID_KEY_PASSWORD.txt` синхронізовані зі storePassword.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Стосується обох застосунків: mlmail (`ANDROID_KEYSTORE_PASSWORD: 7c119980...`) і myshare (`ANDROID_KEYSTORE_PASSWORD: fb0674...`).
- Значення `ANDROID_KEY_PASSWORD` у Infisical (проєкт `vitaliytv-kfse`) потрібно оновити, щоб воно дорівнювало `ANDROID_KEYSTORE_PASSWORD`.
- Keystore-тип: `PKCS12` (підтверджено `keytool -list -v`).
- Локальні копії: `~/keystores-release/<app>/secrets/`.
