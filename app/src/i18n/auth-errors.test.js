import { describe, it, expect } from 'vitest'
import { errorMessage } from './auth-errors.js'

describe('errorMessage', () => {
  it('returns Ukrainian message for Cancelled', () => {
    expect(errorMessage('Cancelled')).toBe('Логін скасовано.')
  })

  it('returns Ukrainian message for Network', () => {
    expect(errorMessage('Network')).toBe("Не вдалося з'єднатися з Google. Перевірте мережу.")
  })

  it('returns Ukrainian message for OAuth', () => {
    expect(errorMessage('OAuth')).toBe('Помилка авторизації Google.')
  })

  it('returns Ukrainian message for Storage', () => {
    expect(errorMessage('Storage')).toBe('Не вдалося зберегти токен у захищене сховище пристрою.')
  })

  it('returns Ukrainian message for ReauthRequired', () => {
    expect(errorMessage('ReauthRequired')).toBe('Сеанс прострочений — увійдіть знову.')
  })

  it('returns Ukrainian message for Platform', () => {
    expect(errorMessage('Platform')).toBe('Помилка платформи.')
  })

  it('falls back to Unknown message for unknown kinds', () => {
    expect(errorMessage('SomethingNotInTable')).toBe('Невідома помилка.')
  })

  it('falls back to Unknown message for null/undefined', () => {
    expect(errorMessage(null)).toBe('Невідома помилка.')
    expect(errorMessage()).toBe('Невідома помилка.')
  })
})
