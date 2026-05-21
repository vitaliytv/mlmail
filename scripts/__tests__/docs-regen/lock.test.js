import { describe, it, expect, beforeEach, afterEach } from 'bun:test'
import { randomUUID } from 'node:crypto'
import { join } from 'node:path'
import { mkdir, rm, access } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { acquireLock } from '../../docs-regen/lock.js'

let LOCK_DIR

beforeEach(async () => {
  LOCK_DIR = join(tmpdir(), `lock-test-${Date.now()}-${randomUUID()}`)
  await mkdir(LOCK_DIR, { recursive: true })
})

afterEach(async () => {
  await rm(LOCK_DIR, { recursive: true, force: true })
})

describe('acquireLock', () => {
  it('creates lock file and returns release fn', async () => {
    const path = join(LOCK_DIR, 'a.lock')
    const lock = await acquireLock(path)
    expect(lock.acquired).toBe(true)
    expect(typeof lock.release).toBe('function')
    // file exists
    await access(path) // throws if missing
    await lock.release()
    // file gone after release
    await expect(access(path)).rejects.toThrow()
  })

  it('returns acquired:false when lock exists', async () => {
    const path = join(LOCK_DIR, 'b.lock')
    const first = await acquireLock(path)
    expect(first.acquired).toBe(true)
    const second = await acquireLock(path)
    expect(second.acquired).toBe(false)
    await first.release()
  })

  it('writes pid into lock file', async () => {
    const path = join(LOCK_DIR, 'c.lock')
    const lock = await acquireLock(path)
    const text = await Bun.file(path).text()
    expect(text).toBe(String(process.pid))
    await lock.release()
  })
})
