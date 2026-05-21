import { createHash } from 'node:crypto'

/**
 * Compute a prefixed SHA-256 digest of the given content.
 * @param {string} content Content to hash.
 * @returns {string} Digest in `sha256:<hex>` form.
 */
export function sha256(content) {
  return 'sha256:' + createHash('sha256').update(content).digest('hex')
}
