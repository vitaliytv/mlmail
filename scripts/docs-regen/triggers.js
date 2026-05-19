const RULE_FILE = '.cursor/rules/n-ci4.mdc'

export function detectTriggers({ adrs, manifest, ruleHash, templateHashes }) {
  const unmarked = adrs.filter((a) => !a.hasMark).map((a) => a.slug)

  const currentSlugs = new Set(adrs.map((a) => a.slug))
  const removed = Object.keys(manifest.adrs || {}).filter((s) => !currentSlugs.has(s))

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
