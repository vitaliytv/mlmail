# Зберігання Android release-keystore для команди через Infisical

**Status:** Accepted
**Date:** 2026-06-19

## Context and Problem Statement

Проєкт mlmail потребує Android release-keystore (`mlmail-release.jks`, alias `mlmail`) для підписання APK/AAB у CI. Втрата ключа або паролів унеможливлює публікацію оновлень у Google Play. Необхідна схема зберігання з командним доступом і RBAC, що не залежить від особистого акаунту одного розробника.

## Considered Options

* Особистий пароль-менеджер одного розробника — немає командного доступу
* 1Password Teams/Business — нативні файл-вкладення, платна SaaS
* Bitwarden Organizations (self-hosted або хмара) — відкритий код, RBAC
* KeePassXC із `.kdbx` у спільному сховищі — потребує окремого сховища файлів
* Infisical SecretOps (хмара або self-host) — RBAC, версіонування, аудит-лог, нативний GitHub Actions Sync

## Decision Outcome

Chosen option: "Infisical SecretOps як робоче джерело + офлайн зашифрований бекап", because Infisical підтримує RBAC, версіонування та нативну синхронізацію в GitHub Actions Secrets; keystore як `ANDROID_KEYSTORE_BASE64` відповідає форматові, що очікує `.github/workflows/release.yml`.

### Consequences

* Good, because доступ не прив'язаний до одного акаунту — керується RBAC Infisical `prod` env.
* Good, because синхронізація в GitHub Actions Secrets автоматична — без ручного `gh secret set`.
* Good, because версіонування дозволяє відкотити секрет при перезаписі.
* Bad, because Infisical зберігає лише key-value рядки — `.jks` у base64 не очевидний для нових членів команди.
* Bad, because втрата доступу до Infisical-інстансу = втрата ключа без офлайн-бекапу.

## More Information

| Роль | Де |
|---|---|
| Майстер-копія | Offline зашифрований бекап (hdiutil + USB) |
| Командне джерело | Infisical `prod` env (SecretOps) |
| CI | GitHub Actions Secrets (синк з Infisical) |

Секрети: `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`. Генерація: `keytool -genkey -v -keystore mlmail-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias mlmail`. Заливання: `base64 -i mlmail-release.jks | infisical secrets set ANDROID_KEYSTORE_BASE64 --env=prod`. Офлайн-бекап: `hdiutil create -encryption -stdinpass -volname keys -srcfolder ./mlmail-release.jks mlmail-keys.dmg`. Документація: `docs/android-release-keystore.md`. Рекомендація: увімкнути Play App Signing при першому завантаженні в Play Console.
