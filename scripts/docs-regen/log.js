import { appendFile, mkdir } from 'node:fs/promises'
import { join, dirname } from 'node:path'

export class Logger {
  constructor(rootDir) {
    this.rootDir = rootDir
    this.buffer = []
  }

  info(msg) {
    const line = `[info ] ${new Date().toISOString()} ${msg}`
    console.log(line)
    this.buffer.push(line)
  }

  warn(msg) {
    const line = `[warn ] ${new Date().toISOString()} ${msg}`
    console.warn(line)
    this.buffer.push(line)
  }

  error(msg) {
    const line = `[error] ${new Date().toISOString()} ${msg}`
    console.error(line)
    this.buffer.push(line)
  }

  async flush() {
    const path = join(this.rootDir, 'docs/ci4/.regen.log')
    await mkdir(dirname(path), { recursive: true })
    await appendFile(path, this.buffer.join('\n') + '\n', 'utf8')
    this.buffer = []
  }
}
