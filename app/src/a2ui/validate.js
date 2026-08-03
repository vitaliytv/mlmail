/** @see ./docs/validate.md */

import { CATALOG_NITRA_CORE, NITRA_CORE_COMPONENTS } from './catalog.js'

const OP_KEYS = [
  'createSurface',
  'updateComponents',
  'updateDataModel',
  'deleteSurface',
  'callFunction',
  'actionResponse'
]

const ALLOWED_PROPS = {
  Text: new Set(['text', 'variant', 'weight']),
  Column: new Set(['children', 'justify', 'align', 'weight']),
  Row: new Set(['children', 'justify', 'align', 'weight']),
  Button: new Set(['child', 'variant', 'action', 'weight']),
  Divider: new Set(['axis', 'weight'])
}

/**
 * Lightweight JS mirror of plugin-a2ui catalog checks for Vue unit tests.
 * Production host should prefer the Rust validator; this rejects unknown
 * catalog refs / components so unrecognized content is never rendered.
 * @param {object[]} messages
 * @returns {{ ok: true, surfaces: Map<string, object> } | { ok: false, error: string }}
 */
export function validateA2uiStream(messages) {
  if (!Array.isArray(messages)) {
    return { ok: false, error: 'messages must be an array' }
  }
  /** @type {Map<string, { catalogId: string, components: Map<string, object>, dataModel: object }>} */
  const surfaces = new Map()

  for (const msg of messages) {
    if (!msg || typeof msg !== 'object') {
      return { ok: false, error: 'message must be object' }
    }
    if (msg.version !== 'v1.0') {
      return { ok: false, error: `version must be v1.0, got ${msg.version}` }
    }
    const ops = OP_KEYS.filter(k => k in msg)
    if (ops.length !== 1) {
      return { ok: false, error: `expected one op, got ${ops.join(',')}` }
    }
    const op = ops[0]
    if (op === 'createSurface') {
      const body = msg.createSurface
      const id = body?.surfaceId
      if (!id) return { ok: false, error: 'createSurface.surfaceId required' }
      if (surfaces.has(id)) return { ok: false, error: `surface exists: ${id}` }
      const catalogId = body.catalogId || CATALOG_NITRA_CORE
      if (catalogId !== CATALOG_NITRA_CORE) {
        return { ok: false, error: `unknown catalog: ${catalogId}` }
      }
      const components = new Map()
      const err = ingestComponents(body.components || [], components, catalogId)
      if (err) return { ok: false, error: err }
      surfaces.set(id, {
        catalogId,
        components,
        dataModel: body.dataModel && typeof body.dataModel === 'object' ? body.dataModel : {}
      })
    } else if (op === 'updateComponents') {
      const body = msg.updateComponents
      const surface = surfaces.get(body?.surfaceId)
      if (!surface) return { ok: false, error: `surface not found: ${body?.surfaceId}` }
      const err = ingestComponents(body.components || [], surface.components, surface.catalogId)
      if (err) return { ok: false, error: err }
    } else if (op === 'updateDataModel') {
      const body = msg.updateDataModel
      const surface = surfaces.get(body?.surfaceId)
      if (!surface) return { ok: false, error: `surface not found: ${body?.surfaceId}` }
      if (!('value' in (body || {}))) return { ok: false, error: 'updateDataModel.value required' }
      const path = body.path || '/'
      if (path !== '/' && path !== '') {
        return { ok: false, error: 'MVP updateDataModel supports only path "/"' }
      }
      surface.dataModel = body.value
    } else if (op === 'deleteSurface') {
      const id = msg.deleteSurface?.surfaceId
      if (!surfaces.delete(id)) return { ok: false, error: `surface not found: ${id}` }
    }
  }

  return { ok: true, surfaces }
}

/**
 * @param {object[]} list
 * @param {Map<string, object>} into
 * @param {string} catalogId
 * @returns {string|null}
 */
function ingestComponents(list, into, catalogId) {
  if (!Array.isArray(list)) return 'components must be array'
  for (const c of list) {
    if (!c?.id || !c?.component) return 'component id/component required'
    if (catalogId !== CATALOG_NITRA_CORE && c.catalogId && c.catalogId !== CATALOG_NITRA_CORE) {
      return `unknown catalog: ${c.catalogId}`
    }
    if (!NITRA_CORE_COMPONENTS.includes(c.component)) {
      return `unknown component: ${c.component}`
    }
    const allowed = ALLOWED_PROPS[c.component]
    for (const key of Object.keys(c)) {
      if (['id', 'component', 'catalogId', 'accessibility'].includes(key)) continue
      if (!allowed.has(key)) return `unknown prop '${key}' on ${c.component}`
    }
    into.set(c.id, c)
  }
  return null
}

/** Fixture mirroring plugin-a2ui/fixtures/sidebar_sample.json */
export const SAMPLE_SIDEBAR_STREAM = [
  {
    version: 'v1.0',
    createSurface: {
      surfaceId: 'sidebar.draft-helper',
      catalogId: 'nitra.core',
      components: [
        {
          id: 'root',
          component: 'Column',
          children: ['title', 'from', 'subject', 'actions'],
          justify: 'start',
          align: 'stretch'
        },
        { id: 'title', component: 'Text', text: 'Draft Helper', variant: 'body' },
        { id: 'from', component: 'Text', text: 'From: a@example.com', variant: 'caption' },
        {
          id: 'subject',
          component: 'Text',
          text: 'Subject: Hello from Gmail',
          variant: 'caption'
        },
        {
          id: 'actions',
          component: 'Row',
          children: ['draft_btn'],
          justify: 'start',
          align: 'center'
        },
        { id: 'draft_label', component: 'Text', text: 'Create draft' },
        {
          id: 'draft_btn',
          component: 'Button',
          child: 'draft_label',
          variant: 'primary',
          action: {
            event: { name: 'createDraft', context: { messageId: 'msg_1' } }
          }
        }
      ],
      dataModel: { messageId: 'msg_1' }
    }
  }
]
