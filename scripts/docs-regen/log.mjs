import { appendFile, mkdir } from 'node:fs/promises'
import { join, dirname } from 'node:path'

/**
 * Buffered logger that mirrors output to the console and a docs-regen log file.
 */
export class Logger {
  /**
   * @param {string} rootDir Repository root directory used to resolve the log file path.
   */
  constructor(rootDir) {
    this.rootDir = rootDir
    this.buffer = []
  }

  /**
   * Log an informational message.
   * @param {string} msg Message text.
   * @returns {void}
   */
  info(msg) {
    const line = `[info ] ${new Date().toISOString()} ${msg}`
    console.log(line)
    this.buffer.push(line)
  }

  /**
   * Log a warning message.
   * @param {string} msg Message text.
   * @returns {void}
   */
  warn(msg) {
    const line = `[warn ] ${new Date().toISOString()} ${msg}`
    console.warn(line)
    this.buffer.push(line)
  }

  /**
   * Log an error message.
   * @param {string} msg Message text.
   * @returns {void}
   */
  error(msg) {
    const line = `[error] ${new Date().toISOString()} ${msg}`
    console.error(line)
    this.buffer.push(line)
  }

  /**
   * Flush all buffered lines to the docs-regen log file.
   * @returns {Promise<void>} Resolves once the buffer is written and cleared.
   */
  async flush() {
    const path = join(this.rootDir, 'docs/ci4/.regen.log')
    await mkdir(dirname(path), { recursive: true })
    await appendFile(path, this.buffer.join('\n') + '\n', 'utf8')
    this.buffer = []
  }
}
