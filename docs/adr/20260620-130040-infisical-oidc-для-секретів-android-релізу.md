---
session: 1f2895bd-6329-4038-b7c6-4fa942487719
captured: 2026-06-20T13:00:40+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/1f2895bd-6329-4038-b7c6-4fa942487719.jsonl
---

Напишу ADR-и напряму, оскільки всі факти вже зафіксовані у транскрипті.

## ADR Infisical OIDC для секретів Android-релізу

## Context and Problem Statement
Для підпису Android APK у CI потрібні секрети (keystore, паролі). Початкова спроба синхронізувати секрети з Infisical у GitHub Actions Secrets через Infisical GitHub-інтеграцію не спрацювала: після налаштування `gh secret list -R vitaliytv/mlmail` повертав порожній результат (Infisical Free обмежує цей шлях).

## Considered Options
* Infisical → GitHub Actions Secrets (push-синк через GitHub-інтеграцію Infisical)
* Infisical OIDC — `Infisical/secrets-action` з `method: 'oidc'` (патерн з репо `nitra`)
* Заливка секретів напряму через `gh secret set` з локальних файлів

## Decision Outcome
Chosen option: "Infisical OIDC (`method: 'oidc'`)", because це вже був робочий патерн у `nitra`-репо (воркфлоу `gt-run.yml`): GitHub видає JWT, Infisical довіряє йому — жодних секретів у GitHub взагалі не потрібно.

### Consequences
* Good, because transcript фіксує очікувану користь: нульові GitHub Actions Secrets у репо, секрети залишаються тільки в Infisical.
* Bad, because вимагає трьох окремих кроків у дашборді Infisical, які CLI не вміє робити: налаштування OIDC Auth на identity, розширення Subject-патерну на `refs/tags/*`, і додавання identity як члена проєкту. Кожен пропущений крок давав окремий 403.

## More Information
- Файл: `.github/workflows/release.yml` (mlmail, myshare) — блок `build-android`, permissions: `id-token: write`, крок `Infisical/secrets-action@v1.0.16`
- Identity ID: `53691c96-17d9-4389-b078-0f77073809ab`, project-slug: `vitaliytv-kfse`, domain: `https://secret.7n.ai`, env-slug: `main`, secret-path: `/mlmail`
- Subject-патерн, що фактично спрацював: `repo:{nitra,vitaliytv}/*:ref:refs/{heads/{dev,main},tags/*}` (вкладені дужки — обидва попередні варіанти без `refs/tags/*` давали 403 на тег-тригері)
- `gh secret list -R vitaliytv/mlmail` повертає порожньо навмисно — це очікуваний стан при OIDC-підході

---

## ADR PKCS12 keystore: keyPassword дорівнює storePassword

## Context and Problem Statement
Gradle падав на пакуванні APK з помилкою `Get Key failed: Given final block not properly padded` навіть після того, як Infisical OIDC запрацював і keystore-файл розшифровувався. Keystore було згенеровано з окремим `-keypass`, відмінним від `-storepass`.

## Considered Options
* Окремі паролі store і key (як згенеровано `keytool -genkey -keypass …`)
* keyPassword = storePassword (пароль ключа збігається з паролем сховища)

## Decision Outcome
Chosen option: "keyPassword = storePassword", because `openssl pkcs12` підтвердив: приватний ключ розшифровується тільки `storePassword` (keys: 1), а окремий `keyPassword` не дає результату (keys: 0). Сучасний `keytool` створює keystore у форматі PKCS12, де пароль ключа ігнорується при генерації — ключ завжди шифрується паролем сховища.

### Consequences
* Good, because усуває залежність від секрету `ANDROID_KEY_PASSWORD` в Infisical — CI читає тільки `ANDROID_KEYSTORE_PASSWORD` і підставляє його на обидві позиції у `keystore.properties`.
* Bad, because `keytool` і власні тести через `-certreq` / `-importkeystore` приймали будь-який пароль ключа (не дискримінували), тому помилку було виявлено лише через `openssl pkcs12`. Це ускладнило локальну діагностику.

## More Information
- Діагностична команда: `openssl pkcs12 -in mlmail-release.jks -passin pass:<SP> -nokeys` і `openssl pkcs12 -in mlmail-release.jks -passin pass:<SP>` — інструмент OpenSSL 3.6.2
- Файл: `.github/workflows/release.yml`, крок «Write keystore.properties»: `echo "keyPassword=$ANDROID_KEYSTORE_PASSWORD"` (замість `$ANDROID_KEYSTORE_PASSWORD` для key і `$ANDROID_KEYSTORE_PASSWORD` для store — роздільний `ANDROID_KEY_PASSWORD` не використовується)
- Keystores: `~/keystores-release/mlmail/mlmail-release.jks`, `~/keystores-release/myshare/myshare-release.jks` — обидва PKCS12
- Те саме правило стосується майбутнього keystore для `task`

---

## ADR `bun --cwd=app` (з `=`) обов'язковий у CI

## Context and Problem Statement
Крок «Build signed APK» завершувався з exit 0 без реальної компіляції — Rust-збірки не відбувалось, файл `.apk` не зʼявлявся, і наступний крок upload падав з «no matches found». Причину виявлено при локальному тестуванні форм виклику `bun`.

## Considered Options
* `bun --cwd app run tauri android build --apk` (пробіл між прапорцем і значенням)
* `bun --cwd=app run tauri android build --apk` (знак `=` між прапорцем і значенням)

## Decision Outcome
Chosen option: "`bun --cwd=app` (з `=`)", because при формі з пробілом `bun` не розпізнає `--cwd` як прапорець, друкує власний usage-текст і виходить з кодом 0 — без запуску скрипту. Підтверджено локально: `bun --cwd app run tauri --version` → `Usage: bun run [flags]...`.

### Consequences
* Good, because transcript фіксує очікувану користь: `bun --cwd=app run tauri --version` → `tauri-cli 2.11.2` локально і реальна 20-хвилинна Rust-компіляція у CI.
* Bad, because помилка silent (exit 0), тому крок «Build signed APK» показував ✓ у GA, і реальна причина failure була прихована.

## More Information
- Файли: `.github/workflows/release.yml` (mlmail, myshare, task — коментар), крок `run: bun --cwd=app run tauri android build --apk`
- Локальна верифікація: `bun --cwd=app run tauri --version` vs `bun --cwd app run tauri --version`

---

## ADR Git-тег як єдине джерело версії артефактів релізу

## Context and Problem Statement
Після першого успішного DMG-білду артефакт у релізі `v0.1.1` називався `mlmail_0.1.0_universal.dmg` — версія бралася з `app/src-tauri/tauri.conf.json` (`0.1.0`), а не з git-тегу `v0.1.1`. Це призводило до невідповідності між назвою релізу і назвою файлу.

## Considered Options
* Версія з `tauri.conf.json` (статична, змінюється вручну)
* Git-тег як джерело версії (перезаписувати `tauri.conf.json` перед збіркою)

## Decision Outcome
Chosen option: "Git-тег як джерело версії", because тег вже є тригером релізу і однозначно ідентифікує версію; статична версія у файлі вимагала б ручного синку при кожному релізі.

### Consequences
* Good, because transcript фіксує очікувану користь: реліз `v0.1.3` містить `mlmail_0.1.3_universal.dmg` і `mlmail_0.1.3_universal.apk` — усі артефакти відповідають тегу.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Файли: `.github/workflows/release.yml` (mlmail, myshare), крок «Sync app version from tag» у `build-desktop` і `build-android`
- Команда: `VER="${GITHUB_REF_NAME#v}"; jq --arg v "$VER" '.version = $v' app/src-tauri/tauri.conf.json > /tmp/tc.json && mv /tmp/tc.json app/src-tauri/tauri.conf.json`
- Крок виконується до `tauri-action` / `tauri android build`, тому версія в зібраному бінарнику і в назві артефакту збігаються

---

## ADR Перейменування APK-артефакту та заборона `ls` у shell-скриптах CI

## Context and Problem Statement
Стандартна Tauri Android збірка виводить файл `app-universal-release.apk` — без назви застосунку і версії. Крім того, перший варіант кроку upload використовував `ls` для пошуку APK, що спричинило помилку `SC2012` від shellcheck у кроці `lint-ga`.

## Considered Options
* Залишити стандартну назву `app-universal-release.apk`
* Перейменувати у `<appname>_<version>_universal.apk` через `find` + `cp` перед upload

## Decision Outcome
Chosen option: "Перейменування через `find` + `cp`", because назва артефакту має збігатися з DMG-конвенцією (`mlmail_<version>_universal.dmg`); `find` замість `ls` вимагає shellcheck (SC2012), що перевіряється в lint-ga CI.

### Consequences
* Good, because transcript фіксує очікувану користь: у релізі `v0.1.3` APK названо `mlmail_0.1.3_universal.apk`, що відповідає DMG.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Файли: `.github/workflows/release.yml` (mlmail, myshare), крок «Upload APK to release»
- Команда: `SRC=$(find app/src-tauri/gen/android/app/build/outputs/apk/universal/release -name '*.apk' | head -1); cp "$SRC" "$RUNNER_TEMP/<appname>_${VER}_universal.apk"; gh release upload "$GITHUB_REF_NAME" "$RUNNER_TEMP/<appname>_${VER}_universal.apk"`
- SC2012 перевіряється через `bun run lint-ga` (actionlint + shellcheck + zizmor)
