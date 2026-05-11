const messages = {
  Cancelled: 'Логін скасовано.',
  Network: "Не вдалося з'єднатися з Google. Перевірте мережу.",
  OAuth: 'Помилка авторизації Google.',
  Storage: 'Не вдалося зберегти токен у захищене сховище пристрою.',
  ReauthRequired: 'Сеанс прострочений — увійдіть знову.',
  Platform: 'Помилка платформи.',
  Unknown: 'Невідома помилка.'
}

export function errorMessage(kind) {
  if (kind === null || kind === undefined) return messages.Unknown
  return messages[kind] ?? messages.Unknown
}
