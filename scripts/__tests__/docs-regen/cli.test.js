import { describe, it, expect } from 'bun:test'
import { parseCliArgs } from '../../docs-regen/cli.js'

describe('parseCliArgs', () => {
  it('defaults: all false, projection undefined', () => {
    const result = parseCliArgs([])
    expect(result.all).toBe(false)
    expect(result.dry).toBe(false)
    expect(result.noMark).toBe(false)
    expect(result.check).toBe(false)
    expect(result.projection).toBeUndefined()
  })

  it('--projection 01-context', () => {
    const result = parseCliArgs(['--projection', '01-context'])
    expect(result.projection).toBe('01-context')
  })

  it('--all sets all=true', () => {
    expect(parseCliArgs(['--all']).all).toBe(true)
  })

  it('--dry sets dry=true', () => {
    expect(parseCliArgs(['--dry']).dry).toBe(true)
  })

  it('--no-mark sets noMark=true', () => {
    expect(parseCliArgs(['--no-mark']).noMark).toBe(true)
  })

  it('--check sets check=true', () => {
    expect(parseCliArgs(['--check']).check).toBe(true)
  })

  it('throws on unknown flag', () => {
    expect(() => parseCliArgs(['--bogus'])).toThrow()
  })
})
