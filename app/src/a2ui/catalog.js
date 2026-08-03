/** @see ./docs/catalog.md */

/** Host catalog id for the MVP Vue A2UI renderer. */
export const CATALOG_NITRA_CORE = 'nitra.core'

/** Component names implemented by the host renderer. */
export const NITRA_CORE_COMPONENTS = Object.freeze(['Text', 'Column', 'Row', 'Button', 'Divider'])

/**
 * Resolve a dynamic A2UI string (literal | {path} | other) against a data model.
 * MVP: only plain strings and `{ path }` bindings; function calls return empty.
 * @param {unknown} value
 * @param {Record<string, unknown>} dataModel
 * @returns {string}
 */
export function resolveDynamicString(value, dataModel = {}) {
  if (typeof value === 'string') return value
  if (value && typeof value === 'object' && 'path' in value) {
    const path = String(/** @type {{ path: string }} */ (value).path || '')
    const key = path.replace(/^\//, '')
    const resolved = key ? dataModel[key] : undefined
    return resolved == null ? '' : String(resolved)
  }
  return ''
}

/**
 * Build an id → component map from a surface state's components object or array.
 * @param {Record<string, object>|object[]} components
 * @returns {Map<string, object>}
 */
export function indexComponents(components) {
  const map = new Map()
  if (Array.isArray(components)) {
    for (const c of components) {
      if (c?.id) map.set(c.id, c)
    }
    return map
  }
  if (components && typeof components === 'object') {
    for (const [id, c] of Object.entries(components)) {
      map.set(id, c)
    }
  }
  return map
}
