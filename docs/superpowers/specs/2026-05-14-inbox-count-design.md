# Кількість листів у скриньці на стартовому екрані — design spec

**Дата:** 2026-05-14
**Статус:** Approved, готовий до плану імплементації
**Scope:** Показати точну кількість листів у мітці INBOX на стартовому екрані MLMaiL після успішного логіну. Без списку листів, без авто-оновлення за таймером, без кнопки «оновити».

## Мета

1. Після успішної Google-авторизації користувач MLMaiL бачить на стартовому екрані рядок «Листів у скриньці: N», де N — точна загальна кількість повідомлень із міткою `INBOX` у Gmail-акаунті.
2. Число фетчиться один раз під час `initialize()` (для збереженої сесії після рестарту) і ще раз одразу після `login()` (для свіжого входу). Без таймерів, без кнопки оновити в цьому ітерейшені.
3. Якщо Gmail API повертає 401 (термін доступу/refresh-токен зіпсувався) — auth-store трактує це як `ReauthRequired` і повертає користувача в стан «не залогінений», як це вже робить `auth_get_access_token`.
4. Інші помилки (мережа, 5xx, парсинг) — показуються українським повідомленням біля рядка «Листів у скриньці», число не показується.

Що **поза scope** цієї ітерації:

- Список листів, відкривання листа, дії над листом.
- Лічильник непрочитаних окремо.
- Авто-оновлення за інтервалом, push-нотифікації, watch API.
- Кнопка «оновити» в UI (можна додати в окремій ітерації).
- Кеш числа в локальному сховищі між запусками.
- Реальний Gmail API виклик у unit-тестах (інтеграційно перевіряється manual checklist'ом).

## Ключові архітектурні рішення

1. **API endpoint:** `GET https://gmail.googleapis.com/gmail/v1/users/me/labels/INBOX`. Точне `messagesTotal` одним викликом. Альтернативи (`getProfile` — по всій скриньці; `messages.list` — `resultSizeEstimate` приблизне) відкинуто свідомо.
2. **HTTP-виклик живе в Rust**, не у Vue. Підтримує наявний патерн (`auth_*`), не виводить access token за межі Rust-процесу, і не вимагає CORS-налаштувань у WebView. Окремий модуль `gmail/` у `app/src-tauri/src/`.
3. **Авторизація запиту:** Rust усередині нової команди викликає внутрішній шлях отримання access token (той самий код, що обслуговує `auth_get_access_token`). На 401 від Gmail — повертає `GmailError::ReauthRequired`, який мапиться у фронт як той самий `kind`, що і з auth (`ReauthRequired`).
4. **Стан у фронті:** число живе у `auth-store.js` як `inboxCount: Ref<number | null>` (null = ще не завантажено / помилка / не залогінений) і `inboxError: Ref<string | null>` (kind помилки для i18n-мапінгу).
5. **UI MLMaiL — українською.** Rust повертає англомовний `kind`, Vue мапить через існуючий `i18n/auth-errors.js` (або новий `gmail-errors.js`, якщо kinds не перекриваються).

## Архітектура

### Rust шар

Новий модуль `app/src-tauri/src/gmail/mod.rs`:

```
gmail/
  mod.rs        // pub команда gmail_inbox_count + публічний reqwest helper
  error.rs      // GmailError { Network, Http(u16), Parse, ReauthRequired, Platform }
```

- `#[tauri::command] pub async fn gmail_inbox_count(app, state) -> Result<u64, GmailError>`:
  1. Викликає внутрішню функцію `acquire_access_token(&app, &state)` (рефакторинг — виносимо тіло `auth_get_access_token` у приватну функцію, яку дзвонять обидва: команда і Gmail-шар).
  2. Робить GET до labels endpoint з `Authorization: Bearer <token>`.
  3. 200 → `serde_json::Value`, дістає `messagesTotal: u64`. Якщо поля немає або не число — `GmailError::Parse`.
  4. 401 → `GmailError::ReauthRequired` + чистимо in-memory token у `state` (як це робить існуючий refresh-шлях).
  5. Будь-яка інша помилка `reqwest` → `Network`. Non-2xx без 401 → `Http(status)`.
- Реєструється в `lib.rs` поряд із `auth::auth_*`.

### Vue шар

`auth-store.js` отримує:

- два нових `ref`: `_inboxCount = ref(null)`, `_inboxErrorKind = ref(null)`;
- метод `async function refreshInboxCount()`:
  - якщо `!_isAuthenticated.value` — нічого;
  - `try`: `_inboxCount.value = await invoke('gmail_inbox_count')`, `_inboxErrorKind.value = null`;
  - `catch (err)`: `_inboxCount.value = null`, `_inboxErrorKind.value = err?.kind ?? 'Unknown'`. Якщо `kind === 'ReauthRequired'` — додатково виставляємо `_isAuthenticated.value = false` і `_email.value = null` (повертаємось у стан логіну);
- `initialize()` після успішної детекції залогіненості викликає `refreshInboxCount()`;
- `login()` після виставляння `_isAuthenticated = true` викликає `refreshInboxCount()`;
- `logout()` додатково ресетить `_inboxCount.value = null`, `_inboxErrorKind.value = null`;
- експорт публічного API доповнюється `inboxCount`, `inboxErrorKind`, `refreshInboxCount`.

`Login.vue`: всередині блоку `signed-in` під рядком «Ви увійшли як …» рендериться:

```vue
<p v-if="auth.inboxCount.value !== null" class="inbox-count">
  Листів у скриньці: {{ auth.inboxCount.value }}
</p>
<p v-else-if="auth.inboxErrorKind.value" class="error">
  {{ errorMessage(auth.inboxErrorKind.value) }}
</p>
<p v-else class="inbox-count muted">Листів у скриньці: …</p>
```

### i18n

Перевикористовуємо існуючу таблицю `app/src/i18n/auth-errors.js`. Kinds, що вже є (`Network`, `ReauthRequired`, `Platform`, `Unknown`) — переюзуємо як є; для Gmail-специфічних додаємо два нових:

- `Http` → «Gmail повернув помилку. Спробуйте пізніше.»
- `Parse` → «Несподівана відповідь від Gmail.»

`Network` повторно використовується для мережевих збоїв і до Gmail (текст «Не вдалося з'єднатися з Google. Перевірте мережу.» лишається валідним). `ReauthRequired` UI обробляє переходом на екран входу, текст самостійно не показується.

## Потік даних (happy path)

1. `App.vue` → `Login.vue.onMounted` → `auth.initialize()`.
2. `auth.initialize()` → `invoke('auth_is_authenticated')` → true → `invoke('auth_current_email')`.
3. `auth.initialize()` → `refreshInboxCount()` → `invoke('gmail_inbox_count')`.
4. Rust: `acquire_access_token` (можливо із рефрешем) → `GET labels/INBOX` → `messagesTotal = N`.
5. Vue: `_inboxCount.value = N` → `Login.vue` рендерить «Листів у скриньці: N».

## Помилки

| Сценарій                                    | Rust kind        | UI                                                                                 |
| ------------------------------------------- | ---------------- | ---------------------------------------------------------------------------------- |
| Немає мережі / DNS / TLS                    | `Network`        | Червоний рядок з повідомленням `Gmail.Network`                                     |
| 401 від Gmail                               | `ReauthRequired` | Повертаємось на екран «Увійти через Google» (показуємо `ReauthRequired` як у auth) |
| 403/429/5xx                                 | `Http`           | Червоний рядок з повідомленням `Gmail.Http`                                        |
| `messagesTotal` відсутнє / не число         | `Parse`          | Червоний рядок з повідомленням `Gmail.Parse`                                       |
| Refresh упав під час `acquire_access_token` | `ReauthRequired` | Повертаємось на екран входу                                                        |

## Тести

**Rust (unit):**

- `gmail::parse_messages_total` (приватна функція в `mod.rs`): парсить валідний JSON; повертає `Parse` без поля; повертає `Parse` для не-числа.
- Status-мапінг: 401 → `ReauthRequired`, 500 → `Http(500)`, network error → `Network` (через `reqwest::Error` мок або шар-обгортка).
- HTTP-виклик до Gmail у unit-тестах не робимо. Інтеграційно перевіряється manual checklist'ом.

**Vue (vitest):**

- `auth-store.test.js`: новий suite «inbox count»:
  - `initialize()` коли користувач залогінений → `inboxCount.value` стає `N` (мокаємо `invoke`).
  - `login()` після успіху → виставляє `inboxCount`.
  - `logout()` → скидає `inboxCount` і `inboxErrorKind`.
  - `gmail_inbox_count` падає з `ReauthRequired` → `isAuthenticated.value = false`, `email.value = null`, `inboxCount.value = null`.
  - `gmail_inbox_count` падає з `Network` → `inboxErrorKind.value = 'Network'`.
- `Login.test.js`: новий тест — після `initialize()` з фейковою сесією і `inboxCount = 42` у DOM зʼявляється «Листів у скриньці: 42».

## Документація, яку оновлюємо

1. `docs/ci4/03-components.md` — додаємо новий компонент `gmail` (Rust) і його контракт із `auth`.
2. `docs/ci4/04-code.md` — нова команда `gmail_inbox_count` у списку контрактів Vue↔Rust.
3. `docs/ci4/decisions.md` — короткий запис рішення «Gmail виклики ходять через Rust, не через WebView fetch».
4. ADR у `docs/adr/_inbox/` — нова нотатка «Inbox count: точне число через `users.labels.get?id=INBOX`».

## Поза scope (для майбутніх ітерацій)

- Кнопка/жест «оновити» поряд із числом.
- Кеш числа на диску, щоб показати останнє значення моментально.
- Лічильник непрочитаних окремо.
- Subscribe на Gmail push (Pub/Sub) — занадто складно для одиничної цифри.
