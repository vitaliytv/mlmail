---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T07:50:39+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Автоматичний реліз Tauri DMG+APK через GitHub Actions з Infisical OIDC

## Context and Problem Statement
Потрібно автоматично збирати та публікувати macOS DMG і Android APK для Tauri-застосунків mlmail і myshare при кожному релізі. Android-підпис потребує секрету keystore, а зберігати його як GitHub Secret означає ручне оновлення при ротації та відсутність єдиного сховища секретів.

## Considered Options
* GitHub Actions Secrets (прямі `${{ secrets.* }}`)
* Infisical Machine Identity з OIDC-авторизацією (без секретів у GitHub репо)

## Decision Outcome
Chosen option: "Infisical Machine Identity з OIDC-авторизацією", because проєкт вже використовує Infisical як єдине сховище секретів (патерн nitra-репо), а OIDC дозволяє CI отримувати JWT від GitHub і підтверджувати автентичність без жодного довгоживучого токена в GitHub.

### Consequences
* Good, because `ANDROID_*`-секрети зберігаються лише в Infisical (`project vitaliytv-kfse`, env `main`, path `/mlmail`); у GitHub Actions Secrets ані один Android-секрет не потрапляє.
* Bad, because OIDC Subject у Infisical-identity мусить явно покривати тег-refs (`repo:vitaliytv/*:ref:refs/tags/*`), що відрізняється від гілок; без цього білд падає з 403 (`Access denied: OIDC subject not allowed`).

## More Information
- `Infisical/secrets-action@v1.0.16` (`method: oidc`, `identity-id: 53691c96-17d9-4389-b078-0f77073809ab`, `project-slug: vitaliytv-kfse`, `env-slug: main`, `secret-path: /mlmail`) — крок у job `build-android`
- Тригер воркфлоу: `on.push.tags: ['v*']` → `sub` токена = `repo:vitaliytv/mlmail:ref:refs/tags/v0.1.1`; Subject у identity має містити `repo:{nitra,vitaliytv}/*:ref:refs/tags/*`
- Файл: `.github/workflows/release.yml` (комміт `6db7343`)

---

## ADR Відсутність rust-cache у release-воркфлоу

## Context and Problem Statement
Під час написання `release.yml` для Tauri-релізу інструмент `zizmor` (аудит workflow, конфіг `.github/zizmor.yml`, `rules.unpinned-uses: ref-pin`) позначив `Swatinem/rust-cache@v2` у release-job як `cache-poisoning (confidence: Low)`.

## Considered Options
* Залишити `Swatinem/rust-cache@v2` для прискорення рідкісних релізних збірок
* Прибрати `rust-cache` з `release.yml`, залишити лише в регулярних lint-воркфлоу

## Decision Outcome
Chosen option: "Прибрати `rust-cache` з `release.yml`", because релізи відбуваються рідко (нема сенсу прогрівати кеш), а залишення cache у release-job ввело б `zizmor`-порушення, яке блокує `bun run lint-ga`.

### Consequences
* Good, because `bun run lint-ga` проходить чисто: `No findings to report. Good job! (31 suppressed)` — zizmor не виводить порушень для `release.yml`.
* Bad, because кожна release-збірка Rust компілюється з нуля; transcript не містить підтверджених негативних наслідків щодо часу білду, але Android Rust-збірка займає ~10–15 хв.

## More Information
- Файл: `.github/workflows/release.yml`; видалено два блоки `Swatinem/rust-cache@v2` (з `build-desktop` і `build-android`)
- Upload заліт через `gh release upload` замість `softprops/action-gh-release@v2`, щоб не вводити зайвий екшен із потенційними cache-poisoning-попередженнями

---

## ADR Конфігурація release-підпису Android у build.gradle.kts з fallback на debug

## Context and Problem Statement
`app/src-tauri/gen/android/app/build.gradle.kts` за замовчуванням не містить release-підпису — APK збирався unsigned або підписувався debug-ключем. CI потребує підписаного APK, але локальна розробка (`tauri android dev`) не повинна ламатись через відсутність keystore.

## Considered Options
* Завжди використовувати keystore (ламає локальний `dev` без файлу `keystore.properties`)
* `signingConfigs.release` з умовним `if (keystorePropertiesFile.exists())` — fallback на debug підпис

## Decision Outcome
Chosen option: "`signingConfigs.release` з умовним fallback на debug підпис", because локальна розробка не вимагає наявності keystore, а CI-середовище записує `keystore.properties` із секретів Infisical перед збіркою.

### Consequences
* Good, because `buildTypes.getByName("release").signingConfig` вибирається динамічно: у CI — release-підпис, локально без файлу — debug. Це зафіксовано як явний патерн у файлі.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Файл: `app/src-tauri/gen/android/app/build.gradle.kts`
- Блок `keystorePropertiesFile.exists()` завантажує `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD` з env-змінних, які інжектує `Infisical/secrets-action`
- Аналогічні зміни застосовані до `myshare/app/src-tauri/gen/android/app/build.gradle.kts`
