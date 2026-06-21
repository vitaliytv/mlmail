---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T08:59:09+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

## ADR Автоматичний реліз DMG + APK через GitHub Actions із тригером на git-тег

## Context and Problem Statement
Проєкти mlmail, myshare і task (Tauri 2 mono-repo з bun-workspaces) не мали механізму автоматичного випуску артефактів. Треба було виробляти `.dmg` (macOS universal) і `.apk` (Android) та прикріплювати їх до GitHub Release за допомогою CI.

## Considered Options
* GitHub Actions workflow із тригером на push git-тегу `v*`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "GitHub Actions workflow із тригером на push git-тегу `v*`", because тег — чіткий, явний сигнал про намір зробити реліз; він несе семантичну версію та ізолює релізні збірки від звичайних push-ів.

### Consequences
* Good, because transcript фіксує очікувану користь: після `git tag vX.Y.Z && git push origin vX.Y.Z` автоматично створюється GitHub Release і до нього прикріплюються `.dmg` і `.apk`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Файл: `.github/workflows/release.yml` (три репо: `vitaliytv/mlmail`, `vitaliytv/myshare`, `nitra/task`)
- Три джоби: `create-release` → `build-desktop` (macOS) і `build-android` (Ubuntu) паралельно
- `build-desktop` використовує `tauri-apps/tauri-action@v0`; upload — `gh release upload` (не `softprops/action-gh-release`)
- `build-android`: `android-actions/setup-android@v3`, `nttld/setup-ndk@v1`, `dtolnay/rust-toolchain@stable`
- Artifact у релізі: `mlmail_0.1.0_universal.dmg` + `mlmail_universal.app.tar.gz` підтверджено наявними у `v0.1.1`

---

## ADR Відмова від `Swatinem/rust-cache` у release-workflow

## Context and Problem Statement
При написанні `release.yml` zizmor (high-confidence) підняв знахідку на `Swatinem/rust-cache`, що за замовчуванням увімкнений кеш і є вектором cache-poisoning у release-контексті.

## Considered Options
* Залишити `rust-cache` із явним `save-always: false`
* Прибрати `rust-cache` повністю
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Прибрати `rust-cache` повністю", because release-збірки рідкісні, а cache-poisoning у release-pipeline несе вищий ризик, ніж приріст у швидкості.

### Consequences
* Good, because zizmor після видалення повернув `No findings to report. Good job! (31 suppressed)` — аудит чистий.
* Bad, because кожна release-збірка компілює Rust з нуля (~10–15 хв).

## More Information
- Команда перевірки: `bun run lint-ga` (actionlint + zizmor)
- Конфіг zizmor: `.github/zizmor.yml`, `rules.unpinned-uses.config.policies['*']: ref-pin`

---

## ADR Infisical OIDC machine identity для Android-секретів у CI

## Context and Problem Statement
Android release-збірка потребує keystore + паролів (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`). Зберігати їх напряму в GitHub Actions Secrets означає ручне копіювання між репо; команда вже використовувала Infisical як єдине джерело правди для секретів.

## Considered Options
* GitHub Actions Secrets (пряме збереження через `gh secret set`)
* Infisical GitHub-інтеграція (push-синк із Infisical у GHA Secrets)
* Infisical OIDC machine identity (`Infisical/secrets-action`, `method: oidc`) — патерн, вже використаний у `nitra/ai`

## Decision Outcome
Chosen option: "Infisical OIDC machine identity", because це вже наявний патерн в org nitra (воркфлоу `nitra/ai/.github/workflows/gt-run.yml`), не потребує жодних секретів у GitHub-репо і дає централізоване управління через Infisical.

### Consequences
* Good, because transcript фіксує очікувану користь: нуль секретів у GitHub, Android-секрети залишаються тільки в Infisical.
* Bad, because OIDC Subject pattern потребував двох виправлень: спочатку паттерни склеїлись (`…{dev,main}repo:…`), потім правильний вираз `repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}` ще не перевірений до кінця (остання спроба в transcript завершилась помилкою 403).

## More Information
- Дія: `Infisical/secrets-action@v1.0.16`, `method: 'oidc'`
- `domain: 'https://secret.7n.ai'`, `project-slug: 'vitaliytv-kfse'`
- `identity-id: '53691c96-17d9-4389-b078-0f77073809ab'`
- `env-slug: 'main'`, `secret-path: '/mlmail'`
- Permission у job: `id-token: write` (обов'язково для OIDC)
- Потрібен Subject `repo:vitaliytv/mlmail:ref:refs/tags/v0.1.1` — відповідає патерну `tags/*`

---

## ADR Окремий keystore на кожен застосунок

## Context and Problem Statement
Потрібно підписувати APK для mlmail і myshare (task ще без Android). Питання — один спільний keystore чи окремий на app.

## Considered Options
* Один спільний keystore для всіх застосунків
* Окремий keystore на кожен застосунок

## Decision Outcome
Chosen option: "Окремий keystore на кожен застосунок", because різні застосунки мають різні Play-лістинги; спільний ключ означає спільний ризик компрометації для всіх.

### Consequences
* Good, because transcript фіксує очікувану користь: незалежне управління ключами та ізоляція ризику між застосунками.
* Bad, because треба зберігати та ротувати окремі keystore-файли і секрети для кожного app.

## More Information
- Keystore генерувались командою `keytool -genkey -v -keystore <app>-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias <app>`
- Файли зберігаються в `~/keystores-release/<app>/` (поза git-репо, `chmod 700`)
- Base64 і паролі — в `~/keystores-release/<app>/secrets/*.txt`
- Infisical secret path: `/mlmail` для mlmail, `/myshare` для myshare (окремі шляхи)

---

## ADR Синхронізація версії застосунку з git-тегом у CI

## Context and Problem Statement
Версія артефакту брала значення з `app/src-tauri/tauri.conf.json` (статичне `0.1.0`), а не з git-тегу. Результат: DMG отримував ім'я `mlmail_0.1.0_universal.dmg` незалежно від тегу релізу.

## Considered Options
* Вручну підіймати `version` у `tauri.conf.json` перед тегом
* Автоматично переписувати `version` в CI зі значення тегу

## Decision Outcome
Chosen option: "Автоматично переписувати `version` в CI зі значення тегу", because тег стає єдиним джерелом правди для версії; виключає розбіжності між git-тегом і `tauri.conf.json`.

### Consequences
* Good, because transcript фіксує очікувану користь: ім'я артефакту і версія застосунку завжди відповідають пушнутому тегу.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Крок `Sync app version from tag` додано в обидві job: `build-desktop` і `build-android`
- Команда: `VER="${GITHUB_REF_NAME#v}"; node -e "…" app/src-tauri/tauri.conf.json` (виконується до `tauri build`)
- Файл: `.github/workflows/release.yml` у `vitaliytv/mlmail`
