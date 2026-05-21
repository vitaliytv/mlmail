const RULE_FILE = '.cursor/rules/n-ci4.mdc'

/**
 * Determine which ADR and projection changes require regeneration.
 * @param {object} root0 Trigger detection inputs.
 * @param {Array<{slug: string, hasMark: boolean}>} root0.adrs Clean ADRs.
 * @param {object} root0.manifest Previously persisted manifest.
 * @param {string} root0.ruleHash Current hash of the CI4 rule file.
 * @param {Record<string, string>} root0.templateHashes Current template hashes by name.
 * @returns {{unmarked: string[], removed: string[], rulesChanged: boolean, templatesChanged: boolean}} Detected triggers.
 */
export function detectTriggers({ adrs, manifest, ruleHash, templateHashes }) {
  const unmarked = adrs.filter(a => !a.hasMark).map(a => a.slug)

  const currentSlugs = new Set(adrs.map(a => a.slug))
  const removed = Object.keys(manifest.adrs || {}).filter(s => !currentSlugs.has(s))

  const rulesChanged = (manifest.rules?.[RULE_FILE]?.hash ?? null) !== ruleHash

  let templatesChanged = false
  const manifestTpl = manifest.templates || {}
  for (const [name, hash] of Object.entries(templateHashes)) {
    if ((manifestTpl[name]?.hash ?? null) !== hash) {
      templatesChanged = true
      break
    }
  }
  if (!templatesChanged) {
    for (const name of Object.keys(manifestTpl)) {
      if (!(name in templateHashes)) {
        templatesChanged = true
        break
      }
    }
  }

  return { unmarked, removed, rulesChanged, templatesChanged }
}
