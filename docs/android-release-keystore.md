# Android release keystore — генерація, зберігання, CI

Інструкція з керування release-ключем для підпису Android-збірок mlmail.

> ⚠️ **Критично:** втрата release-keystore або його паролів = **неможливість оновлювати
> застосунок у Google Play** (Play відхиляє збірки, підписані іншим ключем). Тому ключ
> зберігаємо у трьох місцях: майстер-джерело (Infisical), офлайн-бекап, робоча копія для CI.

---

## 1. Генерація keystore

Виконати **один раз** на старті проєкту. Повторна генерація створює інший ключ — несумісний
з уже опублікованим застосунком.

```bash
keytool -genkey -v -keystore mlmail-release.jks -keyalg RSA -keysize 2048 \
  -validity 10000 -alias mlmail
```

Під час виконання вводимо й **одразу фіксуємо** (знадобляться нижче):

| Значення              | Опис                                  |
|-----------------------|---------------------------------------|
| `storePassword`       | пароль keystore                       |
| `keyPassword`         | пароль ключа (часто = storePassword)  |
| `keyAlias`            | `mlmail`                              |

---

## 2. Майстер-джерело — Infisical

Спільне командне джерело правди. Доступ — лише тим, кому потрібен реліз (RBAC + аудит).

Keystore — бінарний файл, тому в Infisical зберігаємо його як **base64-рядок** (формат, який
уже очікує CI, див. розділ 4).

```bash
# залогінитись і вибрати проєкт mlmail
infisical login

# залити чотири секрети в середовище prod
base64 -i mlmail-release.jks | infisical secrets set ANDROID_KEYSTORE_BASE64 --env=prod
infisical secrets set ANDROID_KEYSTORE_PASSWORD --env=prod
infisical secrets set ANDROID_KEY_ALIAS        --env=prod   # ввести: mlmail
infisical secrets set ANDROID_KEY_PASSWORD     --env=prod
```

Переваги Infisical для команди: рольовий доступ, аудит-логи, версіонування секретів,
self-host або хмара, нативна синхронізація в GitHub Actions.

---

## 3. Офлайн-бекап (обовʼязково)

Secrets-менеджер — для *роздачі* секретів, а не для архіву. Якщо проєкт у Infisical видалять
або зникне доступ до інстансу — ключ пропаде. Тому окремо тримаємо холодну копію.

```bash
# зашифрований .dmg на macOS: усередині сам .jks + текстовий файл із паролями
hdiutil create -encryption -stdinpass -volname mlmail-keys \
  -srcfolder ./keystore-backup mlmail-keys.dmg
```

Зберігати офлайн (USB / сейф), окремо від робочих машин. Файл `.jks` і всі чотири паролі —
завжди разом.

---

## 4. Робоча копія для CI

Воркфлоу `.github/workflows/release.yml` (job `build-android`) очікує **чотири GitHub Secrets**:

- `ANDROID_KEYSTORE_BASE64`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`

На кроці `Configure release signing` він декодує base64 у `keystore.jks` і генерує
`app/src-tauri/gen/android/keystore.properties`.

### Варіант А — синк із Infisical (рекомендовано)

Налаштувати [GitHub-інтеграцію Infisical](https://infisical.com/docs/integrations/cicd/githubactions):
секрети з `prod` автоматично зʼявляються в GitHub Actions Secrets. Оновив у Infisical →
оновилось у CI, без ручних дій.

### Варіант Б — вручну через gh CLI

```bash
base64 -i mlmail-release.jks | gh secret set ANDROID_KEYSTORE_BASE64
gh secret set ANDROID_KEYSTORE_PASSWORD
gh secret set ANDROID_KEY_ALIAS          # mlmail
gh secret set ANDROID_KEY_PASSWORD
```

---

## 5. Play App Signing (рекомендовано)

При першому завантаженні застосунку в Play Console увімкнути
[Play App Signing](https://developer.android.com/studio/publish/app-signing#enroll).
Тоді Google зберігає *app signing key*, а наш `.jks` стає лише *upload key* — його Google
може скинути в разі втрати. Це страховка від найгіршого сценарію.

---

## Підсумок схеми

| Роль            | Де                              | Навіщо                          |
|-----------------|---------------------------------|---------------------------------|
| Майстер-джерело | Infisical (`prod`)              | командний доступ + роздача в CI |
| Офлайн-бекап    | зашифрований `.dmg` / USB       | щоб ніколи не втратити          |
| Робоча копія    | GitHub Secrets (синк з Infisical) | сам білд у `release.yml`       |

**Не зберігати** keystore у git, у месенджерах/пошті, у незашифрованій хмарі чи в
особистому (не командному) акаунті одного розробника.
