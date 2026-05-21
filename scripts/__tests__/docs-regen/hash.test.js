import { describe, it, expect } from 'bun:test'
import { sha256 } from '../../docs-regen/hash.js'

const SHA256_HEX_RE = /^sha256:[0-9a-f]{64}$/

describe('sha256', () => {
  it('returns sha256: prefixed hex for non-empty string', () => {
    const result = sha256('hello')
    expect(result).toBe('sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824')
  })

  it('handles empty string', () => {
    const result = sha256('')
    expect(result).toBe('sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855')
  })

  it('handles unicode (українська)', () => {
    const result = sha256('файловий стор токенів')
    expect(result).toMatch(SHA256_HEX_RE)
  })

  it('produces deterministic output for same input', () => {
    expect(sha256('test')).toBe(sha256('test'))
  })
})
