import { describe, expect, it } from 'vitest'
import { validateInput } from '@7n/tauri-components'
import { getTool, TOOLS } from './catalog.js'

// Domain tests for the local Gmail tool catalog. The generic agent machinery
// (dispatch, manifest, scope, the loop) is tested in @7n/tauri-components; here
// we only cover what's mlmail-specific: the catalog shape, tiers and validators.

describe('catalog', () => {
  it('every tool has a tier, name, summary, input and tauri command', () => {
    for (const tool of TOOLS) {
      expect(['read', 'write', 'destructive']).toContain(tool.tier)
      expect(tool.name).toBeTruthy()
      expect(tool.summary).toBeTruthy()
      expect(tool.input).toBeTypeOf('object')
      expect(tool.tauri).toBeTruthy()
    }
  })

  it('getTool resolves known and unknown names', () => {
    expect(getTool('inbox_count')?.tauri).toBe('gmail_inbox_count')
    expect(getTool('nope')).toBeNull()
  })

  it('tiers the new tools correctly: search/read read, trash destructive, unsubscribe write', () => {
    expect(getTool('search').tier).toBe('read')
    expect(getTool('read').tier).toBe('read')
    expect(getTool('trash').tier).toBe('destructive')
    expect(getTool('unsubscribe').tier).toBe('write')
  })
})

describe('validateInput against catalog tools', () => {
  it('passes when a no-input tool gets nothing', () => {
    expect(validateInput(getTool('inbox_count'))).toBeNull()
  })

  it('search requires a string query', () => {
    expect(validateInput(getTool('search'), {})).toBe('Missing required field: query')
    expect(validateInput(getTool('search'), { query: 5 })).toBe('Field "query" must be a string')
    expect(validateInput(getTool('search'), { query: 'from:bob' })).toBeNull()
  })

  it('read/trash require a string id', () => {
    expect(validateInput(getTool('read'), {})).toBe('Missing required field: id')
    expect(validateInput(getTool('trash'), { id: 'abc' })).toBeNull()
  })

  it('unsubscribe requires an object action', () => {
    expect(validateInput(getTool('unsubscribe'), { action: 'x' })).toBe('Field "action" must be an object')
    expect(validateInput(getTool('unsubscribe'), { action: { OneClick: { url: 'https://x' } } })).toBeNull()
  })
})
