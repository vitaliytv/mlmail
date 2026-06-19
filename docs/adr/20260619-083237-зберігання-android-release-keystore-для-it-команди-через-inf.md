---
session: b5915e92-91d1-4e51-bb40-b735deb1267d
captured: 2026-06-19T08:32:37+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/b5915e92-91d1-4e51-bb40-b735deb1267d.jsonl
---

## ADR Зберігання Android release-keystore для IT-команди через Infisical

## Context and Problem Statement
Проєкт mlmail потребує Android release-keystore (`mlmail-release.jks`) для підписання APK/AAB-збірок у CI. Втрата ключа або паролів робить неможливим публікацію оновлень у Google Play. Необхідно визначити схему зберігання, яка забезпечує командний доступ з контролем прав і не залежить від особистого акаунту одного розробника.

## Considered Options
* Особистий пароль-менеджер (1Password / Bitwarden) одного розробника
* 1Password Teams/Business зі спільним vault
* Bitwarden Organizations (Self-hosted або хмара)
* KeePassXC з `.kdbx`-файлом у спільному сховищі (git / Nextcloud)
* **Infisical** (хмара або self-host) як командний secrets-менеджер

## Decision Outcome
Chosen option: "Infisical як робоче джерело секретів + офлайн зашифрований бекап", because Infisical підтримує RBAC, версіонування секретів, аудит-лог і нативну синхронізацію в GitHub Actions Secrets; keystore зберігається у вигляді base64-рядка (`ANDROID_KEYSTORE_BASE64`), що відповідає форматові, який вже очікує `.github/workflows/release.yml`.

### Consequences
* Good, because секрет не прив'язаний до акаунту однієї людини — доступ налаштовується через RBAC Infisical (`prod` env).
* Good, because Infisical синхронізує секрети в GitHub Actions Secrets автоматично: оновлення в одному місці — без ручного `gh secret set`.
* Good, because версіонування дозволяє відкотити секрет у разі випадкового перезапису.
* Bad, because Infisical зберігає лише key-value рядки (без вкладень файлів), тому `.jks` кодується base64 — не очевидний формат для нових членів команди.
* Bad, because transcript фіксує ризик: якщо Infisical-проєкт буде видалено або втрачено доступ до інстансу, keystore зникне — тому обов'язковий офлайн-бекап.

## More Information
Трирівнева схема зберігання зафіксована в transcript:

| Роль | Де |
|---|---|
| Майстер-копія | Offline зашифрований бекап (`hdiutil create -encryption` / USB) |
| Командне джерело | Infisical `prod` env (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`) |
| CI | GitHub Actions Secrets (синк з Infisical) |

Команда для заливання в Infisical:
```bash
base64 -i mlmail-release.jks | infisical secrets set ANDROID_KEYSTORE_BASE64 --env=prod
```

Документація онбординг-процесу збережена у `docs/android-release-keystore.md`.
Workflow CI, що споживає ці секрети: `.github/workflows/release.yml`.

---

## ADR Тририівнева модель резервування Android release-keystore

## Context and Problem Statement
Android release-keystore є невідновлюваним артефактом: Google Play відхиляє APK/AAB підписані іншим ключем, якщо початковий ключ не зареєстрований через Play App Signing. Команді треба забезпечити надійність зберігання за принципом «3 копії».

## Considered Options
* Одне сховище (лише Infisical або лише GitHub Secrets)
* Три рівні: офлайн-бекап + командний secrets-менеджер + CI-середовище

## Decision Outcome
Chosen option: "три рівні зберігання", because один secrets-менеджер не є архівним сховищем і може бути видалений або стати недоступним; офлайн-бекап покриває сценарій втрати доступу до Infisical і GitHub одночасно.

### Consequences
* Good, because transcript фіксує очікувану користь: навіть при повній втраті доступу до хмарних сервісів майстер-копія (.jks + паролі) зберігається офлайн.
* Good, because Play App Signing (реєстрація при першому завантаженні в Play Console) додатково знижує ризик: Google зберігає app signing key, а ротація upload key можлива через консоль.
* Bad, because transcript не містить підтверджених негативних наслідків — офлайн-бекап потребує дисципліни оновлення при ротації ключа.

## More Information
Рекомендований варіант офлайн-бекапу (macOS):
```bash
hdiutil create -encryption -stdinpass -volname keys \
-srcfolder ./mlmail-release.jks mlmail-keys.dmg
```
Документація: `docs/android-release-keystore.md`.
