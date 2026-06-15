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

  it('returns Ukrainian message for ConfigMissing', () => {
    expect(errorMessage('ConfigMissing')).toBe('Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.')
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

describe('errorMessage Gmail kinds', () => {
  it('returns Ukrainian text for Http kind', () => {
    expect(errorMessage('Http')).toBe('Gmail повернув помилку. Спробуйте пізніше.')
  })

  it('returns Ukrainian text for Parse kind', () => {
    expect(errorMessage('Parse')).toBe('Несподівана відповідь від Gmail.')
  })

  it('returns Ukrainian text for Empty kind', () => {
    expect(errorMessage('Empty')).toBe('Скринька порожня.')
  })
})
