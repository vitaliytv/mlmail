import { rmSync } from 'node:fs'
import { open, readFile, rm } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT_DIR = dirname(dirname(fileURLToPath(import.meta.url)))
const LOCK_PATH = join(ROOT_DIR, '.coverage.lock')
const REENTRANT_ENV = 'MLMAIL_METRICS_LOCK'

/**
 * Check whether a process id is still running.
 * @param {number} pid Process id.
 * @returns {boolean} True if the process exists.
 */
function isProcessAlive(pid) {
  try {
    process.kill(pid, 0)
    return true
  } catch (error) {
    return error.code === 'EPERM'
  }
}

/**
 * Acquire the exclusive metrics lock. A lock left by a dead process is reclaimed.
 * @returns {Promise<boolean>} True when the lock was acquired.
 */
async function acquireLock() {
  try {
    const handle = await open(LOCK_PATH, 'wx')
    await handle.write(String(process.pid))
    await handle.close()
    return true
  } catch (error) {
    if (error.code !== 'EEXIST') throw error
  }
  const holder = Number(await readFile(LOCK_PATH, 'utf8').catch(() => ''))
  if (holder && isProcessAlive(holder)) return false
  // Stale lock — the previous run died without releasing it.
  await rm(LOCK_PATH, { force: true })
  return acquireLock()
}

/**
 * Spawn the wrapped command, forwarding stdio.
 * @param {string[]} command Command and arguments.
 * @param {boolean} ownsLock Whether this process holds the lock.
 * @returns {Promise<number>} The command's exit code.
 */
async function runCommand(command, ownsLock) {
  const child = Bun.spawn(command, {
    cwd: process.cwd(),
    stdin: 'inherit',
    stdout: 'inherit',
    stderr: 'inherit',
    env: ownsLock ? { ...process.env, [REENTRANT_ENV]: '1' } : process.env
  })
  if (ownsLock) {
    for (const signal of ['SIGINT', 'SIGTERM']) {
      process.on(signal, () => {
        child.kill()
        rmSync(LOCK_PATH, { force: true })
        process.exit(1)
      })
    }
  }
  return child.exited
}

/**
 * Acquire the metrics lock (unless an ancestor holds it), run the command, release.
 * @returns {Promise<number>} Process exit code.
 */
async function main() {
  const command = process.argv.slice(2)
  if (command.length === 0) {
    console.error('with-lock: відсутня команда для запуску')
    return 1
  }

  // An ancestor wrapper already holds the lock — run directly (reentrant).
  if (process.env[REENTRANT_ENV]) {
    return runCommand(command, false)
  }

  if (!(await acquireLock())) {
    const holder = (await readFile(LOCK_PATH, 'utf8').catch(() => '?')).trim()
    console.error(`✗ Метрики вже виконуються іншим прогоном (.coverage.lock, pid ${holder}) — перервано.`)
    return 1
  }

  try {
    return await runCommand(command, true)
  } finally {
    await rm(LOCK_PATH, { force: true })
  }
}

try {
  process.exitCode = await main()
} catch (error) {
  console.error(`✗ with-lock: ${error.message}`)
  process.exitCode = 1
}
