---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T11:11:45+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

---

## ADR GitHub Actions release pipeline для Tauri-додатків (DMG + APK)

## Context and Problem Statement
У монорепо mlmail (Tauri 2, bun-workspaces, `app/` + `app/src-tauri/`) не було автоматичного релізу: щоб опублікувати `.dmg` (macOS) та `.apk` (Android), треба було збирати артефакти вручну. Аналогічна ситуація у споріднених репо myshare і task.

## Considered Options
* GitHub Actions workflow з тригером на push git-тегу `v*`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "GitHub Actions workflow з тригером на push git-тегу `v*`", because це єдиний варіант, явно реалізований у сесії: `.github/workflows/release.yml` з джобами `create-release`, `build-desktop` (macOS universal DMG через `tauri-apps/tauri-action@v0`) та `build-android` (APK через `bun --cwd=app run tauri android build --apk`).

### Consequences
* Good, because transcript фіксує очікувану користь: push тегу автоматично публікує `.dmg` та підписаний `.apk` у GitHub Releases без ручних дій.
* Bad, because transcript не містить підтверджених негативних наслідків.

---

## ADR Infisical OIDC замість GitHub Secrets для Android-ключів підпису

## Context and Problem Statement
Android APK потребує чотирьох секретів підпису (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`). У репо `vitaliytv/mlmail` відсутні GitHub Actions Secrets; проєкт вже використовує власний Infisical-інстанс (`secret.7n.ai`). Перший варіант — sync-інтеграція Infisical → GitHub Secrets — не спрацював на Free-плані.

## Considered Options
* Infisical OIDC Machine Identity (`Infisical/secrets-action@v1.0.16`, `method: 'oidc'`) — за зразком наявного nitra-патерну
* Sync-інтеграція Infisical → GitHub Secrets (Варіант А з `docs/android-release-keystore.md`)
* Пряме заповнення GitHub Secrets через `gh secret set`

## Decision Outcome
Chosen option: "Infisical OIDC Machine Identity", because це відповідає наявному nitra-патерну (`d43949cb`/`repo:nitra/*`), не залишає секретів у GitHub і не потребує client-secret — GA отримує JWT, Infisical його довіряє.

### Consequences
* Good, because transcript фіксує очікувану користь: жодних Android-секретів у GitHub Actions Secrets; `gh secret list -R vitaliytv/mlmail` залишається порожнім.
* Bad, because transcript фіксує три послідовні 403 до повного відлагодження: (1) OIDC subject не покривав теги (`refs/tags/*`); (2) identity `53691c96` не була членом проєкту `vitaliytv` в Infisical; (3) Subject-патерни були склеєні без роздільника.

## More Information
- `project-slug: vitaliytv-kfse`, `identity-id: 53691c96-17d9-4389-b078-0f77073809ab`, `env-slug: main`, `secret-path: /mlmail`
- Фінальний Subject-патерн: `repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}`
- Файл: `.github/workflows/release.yml` (mlmail), аналогічно `.github/workflows/release.yml` (myshare)

---

## ADR PKCS12 keystore: keyPassword дорівнює storePassword

## Context and Problem Statement
При генерації keystore через `keytool -genkey … -keypass <окремий_пароль>` на сучасному JDK створюється keystore у форматі PKCS12, який не підтримує окремий пароль ключа. gradle при пакуванні APK (`PackageAndroidArtifact`) не зміг розшифрувати ключ, бо в `keystore.properties` було записано відмінний `keyPassword`.

## Considered Options
* Записати в CI `keyPassword=storePassword` (ігнорувати `ANDROID_KEY_PASSWORD`)
* Оновити `ANDROID_KEY_PASSWORD` в Infisical на значення storePassword

## Decision Outcome
Chosen option: "Записати в CI `keyPassword=storePassword`", because OpenSSL 3.6.2 підтвердив: приватний ключ розшифровується виключно storePassword; keytool ігнорує `-keypass` при створенні PKCS12. Залежність від окремого `ANDROID_KEY_PASSWORD` усунена зі скрипту CI:
```yaml
keyPassword=$ANDROID_KEYSTORE_PASSWORD
```

### Consequences
* Good, because transcript фіксує очікувану користь: пакування APK більше не залежить від значення `ANDROID_KEY_PASSWORD` в Infisical — одним секретом менше, що може розійтися.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Файли: `.github/workflows/release.yml` (mlmail, коміт `7e40401`), `.github/workflows/release.yml` (myshare)
- Перевірка: `openssl pkcs12 -in mlmail-release.jks -passin pass:<storePass> -nocerts` → `keys: 1`; з `keyPass` → `keys: 0`

---

## ADR Автосинк версії tauri.conf.json із git-тегу перед збіркою

## Context and Problem Statement
Версія артефактів (DMG, APK) береться з `app/src-tauri/tauri.conf.json` (`"version": "0.1.0"`), а не з git-тегу. Після релізу `v0.1.1` DMG мав назву `mlmail_0.1.0_universal.dmg` — версії в назві файлу й тегу розходилися.

## Considered Options
* Додати крок «Sync app version from tag» у кожну build-джобу перед збіркою
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Додати крок «Sync app version from tag»", because тег є єдиним джерелом правди для релізного номера; `tauri.conf.json` оновлюється через `jq` зі значення `github.ref_name` перед запуском tauri-cli.

### Consequences
* Good, because transcript фіксує очікувану користь: DMG v0.1.2 отримав правильну назву `mlmail_0.1.2_universal.dmg`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Крок додано в джоби `build-desktop` і `build-android` у `.github/workflows/release.yml`
- Команда: `jq --arg v "${GITHUB_REF_NAME#v}" '.version = $v' app/src-tauri/tauri.conf.json > /tmp/tc.json && mv /tmp/tc.json app/src-tauri/tauri.conf.json`

---

## ADR Відсутність rust-cache у release-workflow

## Context and Problem Statement
Перший варіант `release.yml` містив `Swatinem/rust-cache@v2` для обох build-джоб. Zizmor при локальному лінті (`bun run lint-ga`) підняв high-severity finding про cache-poisoning у release-контексті.

## Considered Options
* Прибрати `rust-cache` з release-workflow
* Залишити `rust-cache` з відповідним `zizmor.yml` suppress

## Decision Outcome
Chosen option: "Прибрати `rust-cache` з release-workflow", because релізи рідкісні, кеш некритичний, а zizmor-попередження у release-контексті є обґрунтованим ризиком безпеки.

### Consequences
* Good, because transcript фіксує очікувану користь: `zizmor` повертає `No findings to report` для `release.yml`.
* Bad, because кожен реліз перекомпільовує Rust з нуля (~19 хв для Android за даними transcript).

## More Information
- Правило zizmor: `cache-poisoning` (audit confidence High) для `Swatinem/rust-cache` у release-джобах
- Файл: `.github/workflows/release.yml`
