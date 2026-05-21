# Google OAuth Client IDs у приватному репозиторії та Internal consent screen

**Status:** Accepted
**Date:** 2026-05-13

## Context and Problem Statement

MLMaiL зберігає три Google OAuth Client IDs (Desktop, Android, Web) у `app/src-tauri/.env`. Початково файл додано до `.gitignore`. Також постало питання налаштування OAuth consent screen: зовнішній тип (External) вимагає проходження Google App Verification для sensitive scope `gmail.modify`, що займає до 30 днів. Проєкт `vitaliytv` знаходиться у Workspace-організації `nitralabs.com`.

## Considered Options

Стосовно зберігання Client IDs у git:

- Тримати `app/src-tauri/.env` у `.gitignore`, ділитися Client IDs поза репо вручну
- Додати `app/src-tauri/.env` у приватний репозиторій
- Зашифрувати `.env` через `git-crypt`

Стосовно OAuth consent screen type:

- External тип із Testing-mode (обмеження 100 Test Users)
- External тип із повним Google App Verification (до 30 днів)
- Internal тип (лише для Workspace-організації, `orgInternalOnly: true`)

## Decision Outcome

Chosen option: "Додати `.env` у приватний репозиторій та Internal consent screen через IAP brand", because Google OAuth Client IDs для нативних desktop/mobile-застосунків є публічними ідентифікаторами — вони передаються відкритим текстом у мережевих запитах і тривіально витягуються з бінарника командою `strings`; Internal consent screen усуває вимогу верифікації для `gmail.modify` при обмеженні логіну командою `nitralabs.com`.

### Consequences

- Good, because ознайомлення нового розробника спрощується — Client IDs доступні з репо без окремого кроку передачі.
- Good, because Internal consent screen (`orgInternalOnly: true`) знімає 100-user cap і вимогу Google App Verification для sensitive scopes на ранніх етапах.
- Bad, because Internal consent screen обмежує логін лише акаунтами `*@nitralabs.com`; при масштабуванні за межі організації потрібен перехід на External + verification.
- Neutral, because у `.env` зберігаються лише Client IDs без Client Secret — Google не розглядає native-app Client ID як секрет у своїй моделі безпеки.

## More Information

Процедура створення Internal OAuth consent screen (актуально до 19 березня 2026 — IAP OAuth Admin API відключено; після цієї дати — лише через Google Auth Platform UI `https://console.cloud.google.com/auth/`):

```sh
gcloud organizations list
gcloud projects describe vitaliytv
gcloud iap oauth-brands create \
  --application_title="MLMaiL" \
  --support_email="<email>"
```

Три OAuth Client IDs (Desktop application, Android, Web application) — лише через UI `https://console.cloud.google.com/apis/credentials`.
Scopes (`openid`, `email`, `gmail.modify`) — через Google Auth Platform UI → Data Access.

Файл `.env` зафіксовано у приватний репозиторій `vitaliytv/mlmail` після прибирання рядка з `.gitignore`. Репозиторій приватний (`isPrivate: true`).

Зачіпає: `.gitignore`, `app/src-tauri/.env`, `app/src-tauri/.env.example`, Google Cloud project `vitaliytv`.

Перемикання на External + verification — явна майбутня дія при масштабуванні за межі `nitralabs.com`.

---

**Опрацьовано** 2026-05-20. Проекції:

- [01-context](../ci4/01-context.md)
- [02-containers](../ci4/02-containers.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
