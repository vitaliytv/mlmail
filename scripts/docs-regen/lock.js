import { open, unlink, mkdir } from 'node:fs/promises'
import { dirname } from 'node:path'

export async function acquireLock(path) {
  await mkdir(dirname(path), { recursive: true })
  let handle
  try {
    handle = await open(path, 'wx')
  } catch (error) {
    if (error.code === 'EEXIST') return { acquired: false }
    throw error
  }
  await handle.write(String(process.pid))
  await handle.close()
  return {
    acquired: true,
    release: async () => {
      await unlink(path).catch((error) => {
        if (error.code !== 'ENOENT') throw error
      })
    },
  }
}
