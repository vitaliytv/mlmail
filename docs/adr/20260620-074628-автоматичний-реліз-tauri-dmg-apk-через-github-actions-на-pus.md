---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T07:46:28+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Автоматичний реліз Tauri DMG/APK через GitHub Actions на push тегу

## Context and Problem Statement
Проєкти mlmail, myshare та task потребують механізму публікації артефактів (macOS DMG і Android APK) без ручного збирання. Попередньо жодного release-workflow не існувало — артефакти не збирались і не завантажувались до GitHub Releases.

## Considered Options
* GitHub Actions workflow із тригером на push git-тегу `v*`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "GitHub Actions workflow із тригером на push git-тегу `v*`", because тег однозначно сигналізує про навмисний реліз і дає стабільний ref для іменування артефактів; цей патерн відповідає наявним конвенціям `.github/workflows/` у проєкті.

### Consequences
* Good, because transcript фіксує очікувану користь: один `git push origin vX.Y.Z` запускає три паралельні джоби (`create-release`, `build-desktop`, `build-android`) без ручного втручання.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Файл `.github/workflows/release.yml` (три джоби: `create-release`, `build-desktop` — macOS universal DMG через `tauri-action`, `build-android` — APK через `tauri android build`). Перша перевірка: тег `v0.1.1` → `gh run list` підтвердив старт run 27860408859. Репо task залишено desktop-only (APK вимкнено), бо `app/src-tauri/gen/android` не ініціалізовано.

---

## ADR Infisical OIDC замість GitHub Secrets для Android-підпису

## Context and Problem Statement
Android APK потребує keystore та паролів (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`). Потрібно безпечно подати ці секрети в CI, не зберігаючи їх напряму в GitHub Actions Secrets.

## Considered Options
* GitHub Actions Secrets (пряме зберігання в репо)
* Infisical GitHub-інтеграція (push-синк із Infisical → GitHub Secrets)
* Infisical OIDC machine identity (`Infisical/secrets-action`, метод `oidc`) — nitra-патерн

## Decision Outcome
Chosen option: "Infisical OIDC machine identity", because у проєктах org `nitra` цей патерн вже реалізовано (`Infisical/secrets-action@v1.0.16`, `method: 'oidc'`) і GitHub Actions Secrets у репо залишаються порожніми — секрети тягнуться безпосередньо з Infisical під час збірки за OIDC JWT.

### Consequences
* Good, because transcript фіксує очікувану користь: жодних Android-секретів у GitHub; ротація/оновлення секретів відбувається лише в Infisical без зміни workflow.
* Bad, because transcript фіксує реальний негативний наслідок: перший запуск дав `403 Access denied: OIDC subject not allowed` — OIDC Subject filter identity `53691c96` покривав лише `refs/heads/{dev,main}`, але не `refs/tags/v*`; потребує додавання патерну `repo:{nitra,vitaliytv}/*:ref:refs/tags/*` у дашборді Infisical.

## More Information
Параметри у `release.yml` → job `build-android`: `domain: 'https://secret.7n.ai'`, `project-slug: 'vitaliytv-kfse'`, `identity-id: '53691c96-17d9-4389-b078-0f77073809ab'`, `env-slug: 'main'`, `secret-path: '/mlmail'`. Permission на джобі: `id-token: write`. Зразок nitra: `.github/workflows/gt-run.yml` репо `nitra/ai`. Помилка CI: run 27860408859, job 82455366208, крок `Fetch Android secrets from Infisical`.

---

## ADR Відмова від `rust-cache` у release-workflow

## Context and Problem Statement
При першому запуску `lint-ga` (actionlint + zizmor) workflow містив `Swatinem/rust-cache@v2` для прискорення Rust-збірки. Zizmor повернув high-confidence попередження про cache-poisoning.

## Considered Options
* Залишити `Swatinem/rust-cache` для пришвидшення збірки
* Прибрати `rust-cache` у release-контексті

## Decision Outcome
Chosen option: "Прибрати `rust-cache` у release-контексті", because релізи рідкісні й кеш не є критичним для них; cache-poisoning у release-джобах несе вищий ризик, ніж втрата швидкості.

### Consequences
* Good, because `bun run lint-ga` після видалення повернув `No findings to report. Good job!` для `release.yml`.
* Bad, because Neutral, because transcript не містить підтвердження наслідку щодо реального часу збірки без кешу.

## More Information
Zizmor-знахідка (до виправлення): `Swatinem/rust-cache@v2 enables caching by default, audit confidence → Low`. Конфіг: `.github/zizmor.yml` (`unpinned-uses: ref-pin`, `template-injection: disable`). Після видалення `rust-cache` upload замінено на `gh release upload` замість `softprops/action-gh-release`.

---

## ADR Окремий keystore на кожен застосунок

## Context and Problem Statement
Три застосунки (mlmail, myshare, task) потребують підпису Android APK. Постало питання: один спільний keystore чи окремий на кожен застосунок.

## Considered Options
* Один спільний keystore для всіх застосунків
* Окремий keystore для кожного застосунку

## Decision Outcome
Chosen option: "Окремий keystore для кожного застосунку", because застосунки публікуються під різними Google Play-лістингами (`com.vitaliytv.mlmail`, `com.vitaliytv.myshare`), а різні лістинги не можуть ділити один ключ підпису.

### Consequences
* Good, because transcript фіксує очікувану користь: компрометація ключа одного застосунку не впливає на інші; незалежний цикл ротації.
* Bad, because Neutral, because transcript не містить підтвердження наслідку щодо операційного навантаження.

## More Information
Keystore-файли збережено у `~/keystores-release/<app>/<app>-release.jks` (поза git-репо). Значення секретів — у `~/keystores-release/<app>/secrets/*.txt`. Генерація: `keytool -genkey -v -keyalg RSA -keysize 2048 -validity 10000` через `/Applications/Android Studio.app/Contents/jbr/Contents/Home/bin/keytool`. Для `task` keystore не генерувався — Android-проєкт там не ініціалізовано.
