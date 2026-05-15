const messages = {
  Cancelled: 'Логін скасовано.',
  Network: "Не вдалося з'єднатися з Google. Перевірте мережу.",
  OAuth: 'Помилка авторизації Google.',
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
  if (kind === null || kind === undefined) return messages.Unknown
  return messages[kind] ?? messages.Unknown
}
