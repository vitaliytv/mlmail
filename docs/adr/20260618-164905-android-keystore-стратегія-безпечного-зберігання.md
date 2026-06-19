---
session: b5915e92-91d1-4e51-bb40-b735deb1267d
captured: 2026-06-18T16:49:05+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/b5915e92-91d1-4e51-bb40-b735deb1267d.jsonl
---

## ADR Android keystore — стратегія безпечного зберігання

## Context and Problem Statement
Команда генерує Android release keystore (`mlmail-release.jks`, alias `mlmail`) і потребує надійної схеми зберігання, щоб не втратити доступ до підпису застосунку. CI вже налаштований у `.github/workflows/release.yml` та очікує keystore через GitHub Secrets.

## Considered Options
* Трирівнева схема: менеджер паролів (майстер) + GitHub Secrets (CI) + офлайн зашифрований бекап
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Трирівнева схема зберігання keystore", because втрата `.jks` або паролів унеможливлює публікацію оновлень у Google Play, а `release.yml` вже передбачає передачу keystore через `ANDROID_KEYSTORE_BASE64` як GitHub Secret.

### Consequences
* Good, because transcript фіксує очікувану користь: CI отримує keystore через `ANDROID_KEYSTORE_BASE64` без потрапляння файлу в репозиторій; менеджер паролів зберігає файл і всі чотири значення (`storePassword`, `keyAlias`, `keyPassword`, дату генерації) разом і зашифровано.
* Bad, because transcript не містить підтверджених негативних наслідків; офлайн бекап потребує регулярного оновлення вручну.

## More Information
- Файл keystore: `mlmail-release.jks`, alias `mlmail`, RSA 2048 bit, validity 10000 днів
- Команда генерації: `keytool -genkey -v -keystore mlmail-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias mlmail`
- CI secrets (вже визначені у `release.yml`): `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`
- Завантаження в CI: `base64 -i mlmail-release.jks | pbcopy` → `gh secret set ANDROID_KEYSTORE_BASE64`
- Офлайн бекап: `hdiutil create -encryption -stdinpass -volname keys -srcfolder ./mlmail-release.jks mlmail-keys.dmg`
- Додаткова рекомендація з transcript: увімкнути Play App Signing при першому завантаженні в Play Console, щоб Google зберігав *app signing key*, а ризик обмежувався лише *upload key*
