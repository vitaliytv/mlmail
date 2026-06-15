const messages = {
  Cancelled: 'Логін скасовано.',
  Network: "Не вдалося з'єднатися з Google. Перевірте мережу.",
  OAuth: 'Помилка авторизації Google.',
  ConfigMissing: 'Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.',
  Storage: 'Не вдалося зберегти токен у захищене сховище пристрою.',
  ReauthRequired: 'Сеанс прострочений — увійдіть знову.',
  Platform: 'Помилка платформи.',
  Http: 'Gmail повернув помилку. Спробуйте пізніше.',
  Parse: 'Несподівана відповідь від Gmail.',
  Empty: 'Скринька порожня.',
  Unknown: 'Невідома помилка.'
}

/**
 * @param {string|null} kind error kind code
 * @returns {string} localized error message
 */
export function errorMessage(kind) {
  return messages[kind] ?? messages.Unknown
}
