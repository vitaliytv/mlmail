---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T08:59:14+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Автоматичний реліз Tauri-застосунків через GitHub Actions (DMG + APK)

## Context and Problem Statement
Проєкти `mlmail`, `myshare` та `task` (Tauri 2, bun-монорепо) не мали механізму автоматичного збирання та публікації артефактів. Потрібно було організувати реліз DMG (macOS) і APK (Android) при пуші git-тегу без ручних кроків.

## Considered Options
* Тригер на push тегу `v*` → GitHub Release → паралельні jobs build-desktop і build-android
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Тригер на push тегу `v*` → GitHub Release → паралельні jobs", because це стандартний патерн для Tauri-застосунків: `create-release` → `build-desktop` (`tauri-action`) → `build-android` (`tauri android build`); artifacts заливаються через `gh release upload`.

### Consequences
* Good, because transcript фіксує очікувану користь: DMG `mlmail_0.1.0_universal.dmg` успішно зʼявився в релізі `v0.1.1` після першого запуску `build-desktop`.
* Bad, because `rust-cache` свідомо прибрано (zizmor попередив про cache-poisoning у release-контексті) — збірка довша, але безпечніша.

## More Information
- `.github/workflows/release.yml` у `vitaliytv/mlmail`, `vitaliytv/myshare`, `nitra/task`
- `build-desktop` використовує `tauri-apps/tauri-action@v0`; upload через `gh release upload`
- `build-android` target-архітектури: `aarch64-linux-android,armv7-linux-androideabi,i686-linux-android,x86_64-linux-android`
- `task` (nitra) — desktop-only, бо `app/src-tauri/gen/android` не ініціалізований

---

## ADR Android release-підпис у `build.gradle.kts` з fallback на debug

## Context and Problem Statement
Tauri-генерований `build.gradle.kts` не містив конфігурації release-підпису — APK збирався без підпису. Локальний `tauri android dev` не повинен ламатись за відсутності keystore.

## Considered Options
* `signingConfigs.release` з умовою: якщо `keystore.properties` існує — читати зі змінних середовища, інакше падати на debug-підпис
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "signingConfigs.release з умовою", because це дозволяє і локальну розробку без keystore, і CI-підпис з ключем; `keystore.properties` вже в `.gitignore` проєктів.

### Consequences
* Good, because transcript фіксує очікувану користь: локальний `tauri android build` не ламається за відсутності keystore.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Файли: `app/src-tauri/gen/android/app/build.gradle.kts` у `mlmail` і `myshare`
- Змінні середовища: `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`
- Keystores згенеровано локально: `~/keystores-release/mlmail/`, `~/keystores-release/myshare/`; 10 000 днів, RSA 2048

---

## ADR Infisical OIDC machine identity замість GitHub Secrets для Android-ключів

## Context and Problem Statement
Android keystore та паролі потрібно передавати в CI безпечно. Прямий запис у GitHub Actions Secrets означав би дублювання секретів окремо в кожному репо (витліл + відсутність аудиту).

## Considered Options
* Infisical OIDC machine identity (`method: 'oidc'`, `id-token: write`) — секрети тільки в Infisical, жодних GitHub Secrets
* Infisical Universal Auth (Client ID + Client Secret у GitHub Secrets) — потрібні 2 секрети на репо
* Прямо в GitHub Actions Secrets (`secrets.ANDROID_*`) — початковий варіант, відкинутий після ознайомлення з nitra-патерном

## Decision Outcome
Chosen option: "Infisical OIDC machine identity", because nitra-репо вже використовують цей патерн (`Infisical/secrets-action@v1.0.16`, `method: 'oidc'`) — нуль GitHub Secrets, єдине джерело правди.

### Consequences
* Good, because transcript фіксує очікувану користь: після виправлення OIDC Subject і Project membership жодних секретів у репо немає; `gh secret list -R vitaliytv/mlmail` повертає порожньо.
* Bad, because потребує двох кроків конфігурації в дашборді Infisical (OIDC Subject + Project Membership), які CLI не підтримує і щодо яких виникли помилки під час відлагодження.

## More Information
- `Infisical/secrets-action@v1.0.16`, `method: 'oidc'`, `domain: 'https://secret.7n.ai'`
- `project-slug: 'vitaliytv-kfse'`, `identity-id: '53691c96-17d9-4389-b078-0f77073809ab'`
- `env-slug: 'main'`, `secret-path: '/mlmail'`
- OIDC Subject: `repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}`
- Збірка `permissions.id-token: write` обовʼязкова для OIDC JWT
- Помилки під час відлагодження: спочатку відсутній tag-pattern у Subject (`refs/tags/*`), потім відсутній Project Membership (identity не додана в проєкт)

---

## ADR Синхронізація версії застосунку з git-тегом перед збіркою

## Context and Problem Statement
Версія в `tauri.conf.json` (`0.1.0`) не збігалася з версією git-тегу (`v0.1.1`), тому артефакти іменувалися за `tauri.conf.json`, а не за тегом релізу.

## Considered Options
* Крок «Sync app version from tag» у кожній збірній job: `VER=${GITHUB_REF_NAME#v}; jq --arg v "$VER" '.version=$v' tauri.conf.json` → записати назад перед `tauri build`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Крок Sync app version from tag", because тег стає єдиним джерелом правди — версія застосунку, імена артефактів і реліз завжди збігаються без ручного оновлення `tauri.conf.json`.

### Consequences
* Good, because transcript фіксує очікувану користь: `mlmail_0.1.0_universal.dmg` (з поточного тегу) буде перейменовано в `mlmail_0.1.1_universal.dmg` після наступного тегу, автоматично.
* Bad, because Neutral, because transcript не містить підтвердження наслідку — крок додано до `release.yml` але ще не запущено з новим тегом.

## More Information
- Крок вставлено в обидві job (`build-desktop`, `build-android`) у `.github/workflows/release.yml`
- Команда: `VER=${GITHUB_REF_NAME#v}; jq --arg v "$VER" '.version=$v' app/src-tauri/tauri.conf.json > /tmp/t.json && mv /tmp/t.json app/src-tauri/tauri.conf.json`
- Зміна ще не закомічена (додано під час сесії, але коміт не виконувався)
