---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T07:41:09+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Автоматичний реліз DMG і APK через GitHub Actions на push тегу

## Context and Problem Statement
У проєктах `mlmail`, `myshare` і `task` (Tauri 2 bun-монорепо) не було жодного механізму автоматичного завантаження артефактів у GitHub Releases. Потрібно було зробити так, щоб `.dmg` (macOS) і `.apk` (Android) зʼявлялися в Releases автоматично без ручного завантаження.

## Considered Options
* Тригер на push тегу `v*` з трьома джобами: `create-release`, `build-desktop`, `build-android`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Тригер на push тегу `v*` з трьома джобами", because це стандартна конвенція: гілковий push запускає лінт/тести, а реліз — тільки тег, що відсікає зайві збірки.

### Consequences
* Good, because transcript фіксує очікувану користь: `create-release` відпрацював за 5 с, `build-desktop` і `build-android` стартували паралельно — тег `v0.1.1` одразу запустив реальний Release.
* Bad, because `task` (org `nitra`) отримав desktop-only `.github/workflows/release.yml` (лише DMG) — `gen/android` там не ініціалізований, `tauri android init` ще не виконувався.

## More Information
Файли: `.github/workflows/release.yml` в `vitaliytv/mlmail`, `vitaliytv/myshare`, `nitra/task`. Commit `6db7343` (mlmail). Джоба `build-desktop` використовує `tauri-action` з `targets: aarch64-apple-darwin,x86_64-apple-darwin`; `build-android` — `tauri android build` на `ubuntu-latest` з JDK 17 + Android SDK + NDK r27.

---

## ADR Android release-підпис у build.gradle.kts із fallback на debug

## Context and Problem Statement
Сгенерований Tauri `app/src-tauri/gen/android/app/build.gradle.kts` не мав конфігурації release-підпису — `buildTypes.release` не мав `signingConfig`. CI-збірка APK виходила б unsigned або з debug-сертифікатом, непридатним для Google Play.

## Considered Options
* Додати `signingConfigs.create("release")` з читанням `keystore.properties` і fallback на debug-підпис коли файл відсутній
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "signingConfigs з fallback", because локальний `tauri android dev`/`build` не ламається (keystore.properties немає — підписується debug-ключем), а CI підхоплює реальний keystore через змінні середовища, прописані в `keystore.properties`.

### Consequences
* Good, because transcript фіксує очікувану користь: linter для gradle не скаржився, обидва проєкти (`mlmail`, `myshare`) отримали однакову конфігурацію.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Файли: `app/src-tauri/gen/android/app/build.gradle.kts` у `vitaliytv/mlmail` і `vitaliytv/myshare`. Змінні середовища: `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`. Keystore згенеровано через `keytool` з Android Studio JRE; зберігається в `~/keystores-release/<app>/` поза git-репозиторіями.

---

## ADR Infisical OIDC замість GitHub Actions Secrets для Android-ключів у CI

## Context and Problem Statement
Спочатку планувалося зберігати 4 Android-секрети (`ANDROID_KEYSTORE_BASE64` тощо) безпосередньо в GitHub Actions Secrets. Після того як зʼясувалося, що в проєктах використовується Infisical (`secret.7n.ai`) і там вже є налаштована GitHub-інтеграція для org `nitra` (через `Infisical/secrets-action` method=oidc), виникло питання, як налаштувати аналогічний підхід для org `vitaliytv` без зберігання секретів у GitHub взагалі.

## Considered Options
* `Infisical/secrets-action@v1.0.16` з `method: 'oidc'` (OIDC machine identity, без жодних GH-секретів)
* Infisical GitHub Sync integration (синк секретів у GitHub Actions Secrets через дашборд)
* `gh secret set` напряму з локального keychain через `infisical secrets get`

## Decision Outcome
Chosen option: "`Infisical/secrets-action@v1.0.16` з `method: 'oidc'`", because саме такий патерн вже використовується в org `nitra` (workflow `gt-run.yml`, identity `d43949cb`), і він не вимагає зберігати жодних секретів у GitHub.

### Consequences
* Good, because transcript фіксує очікувану користь: `gh secret list -R vitaliytv/mlmail` повертає порожньо — GitHub Actions Secrets залишились чистими, усе йде через Infisical OIDC.
* Bad, because Neutral, because transcript не містить підтвердження наслідку — на момент закінчення transcript крок `Fetch Android secrets from Infisical` у `build-android` ще не завершився (фоновий монітор ще чекав).

## More Information
Параметри: `domain: 'https://secret.7n.ai'`, `project-slug: 'vitaliytv-kfse'`, `identity-id: '53691c96-17d9-4389-b078-0f77073809ab'`, `env-slug: 'main'`, `secret-path: '/mlmail'`. Permission у джобі: `id-token: write`. Файл: `.github/workflows/release.yml` в `vitaliytv/mlmail`, commit `6db7343`. Конфіг аутентифікації: `~/.infisical/infisical-config.json`, `loggedInUserDomain: https://secret.7n.ai/api`.

---

## ADR Відмова від `Swatinem/rust-cache` і `softprops/action-gh-release` у release-джобах

## Context and Problem Statement
Після першого запуску `bun run lint-ga` (actionlint + zizmor) на `release.yml` zizmor видав high-severity finding на `Swatinem/rust-cache` у release-джобах (cache-poisoning) і скаргу на `softprops/action-gh-release`. Потрібно було усунути находки, не порушуючи функціональності.

## Considered Options
* Видалити `Swatinem/rust-cache` з release-джоб; замінити `softprops/action-gh-release` на `gh release upload`
* Додати zizmor-suppress для cache-poisoning finding
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Видалити rust-cache; використати `gh release upload`", because релізи рідкісні — кеш не критичний для їх швидкості, а `gh` вже є в середовищі runner. Після правки zizmor видав `No findings to report. Good job! (31 suppressed)`.

### Consequences
* Good, because transcript фіксує очікувану користь: `zizmor` і `actionlint` пройшли чисто, CI (Lint GA) підтвердив це на живому пуші (`🌈 completed ./.github/workflows/release.yml`).
* Bad, because реліз-збірки Rust компілюватимуться без кешу — cold build довший (~10–15 хв). Transcript визнає це прийнятним через рідкість релізів.

## More Information
Команда для локальної перевірки: `bun run lint-ga`. Zizmor-конфіг: `.github/zizmor.yml`, правило `unpinned-uses: ref-pin`. Commit, де правка набрала чинності: `6db7343` (`vitaliytv/mlmail`). Завантаження артефактів: `gh release upload "$TAG" "$FILE"` у кроці після `tauri android build`.
