import { writeFile, mkdir } from 'node:fs/promises'
import { join } from 'node:path'
import { callLlm, parseLlmResponse } from './llm.js'

/**
 * Regenerate a single CI4 projection file via the LLM, with one retry on parse failure.
 * @param {object} root0 Projection regeneration parameters.
 * @param {string} root0.name Projection name (e.g. `01-context`).
 * @param {Array<object>} root0.adrs Clean ADRs to feed into the prompt.
 * @param {string} root0.currentContent Current projection file content.
 * @param {Record<string, string>} root0.templates Loaded prompt templates by file name.
 * @param {string} [root0.model] LLM model override.
 * @param {string} root0.rootDir Repository root directory (used for debug dumps).
 * @returns {Promise<{content: string, used_adrs: string[], prompt_length: number, output_length: number}>} Regeneration result.
 */
export async function regenerateProjection({ name, adrs, currentContent, templates, model, rootDir }) {
  const prompt = buildPrompt({ name, adrs, currentContent, templates })

  let raw = await callLlm(prompt, { model })
  let parsed
  try {
    parsed = parseLlmResponse(raw)
  } catch (error) {
    // Save raw output for debugging
    if (rootDir) {
      await saveDebug(rootDir, name, 'attempt1', raw)
    }
    // Retry with a stricter wrapper instruction prepended to the LLM call.
    const retryPrompt =
      prompt +
      '\n\n# Retry-вказівка\n\n' +
      'Попередня відповідь не містила валідного поля `content`. ' +
      'Поверни ВИКЛЮЧНО один валідний JSON-обʼєкт без markdown-fence, без преамбули і без коментарів. ' +
      'Структура: {"content":"<повний markdown>","used_adrs":["<slug>",...]}. ' +
      'У полі content — стрічка з повним markdown-файлом.'
    raw = await callLlm(retryPrompt, { model })
    if (rootDir) {
      await saveDebug(rootDir, name, 'attempt2', raw)
    }
    try {
      parsed = parseLlmResponse(raw)
    } catch (retryError) {
      throw new Error(
        `Projection ${name}: LLM response could not be parsed after retry. ` +
          `First: ${error.message}. Second: ${retryError.message}. ` +
          `Raw outputs saved to docs/ci4/.regen-debug/`,
        { cause: retryError }
      )
    }
  }

  return {
    content: parsed.content,
    used_adrs: parsed.used_adrs,
    prompt_length: prompt.length,
    output_length: raw.length
  }
}

/**
 * Build the full LLM prompt for a projection.
 * @param {object} root0 Prompt parameters.
 * @param {string} root0.name Projection name.
 * @param {Array<object>} root0.adrs Clean ADRs to include in the prompt.
 * @param {string} root0.currentContent Current projection file content.
 * @param {Record<string, string>} root0.templates Loaded prompt templates by file name.
 * @returns {string} Assembled prompt text.
 */
function buildPrompt({ name, adrs, currentContent, templates }) {
  const globalRules = templates['_global.prompt.md']
  const projectionTemplate = templates[`${name}.prompt.md`]

  const adrSection = adrs.map(a => `### ADR: ${a.slug}\n\n${stripExistingMark(a.body)}`).join('\n\n')

  return [
    '# Глобальні правила оформлення',
    '',
    globalRules,
    '',
    '---',
    '',
    `# Інструкції для проекції: ${name}`,
    '',
    projectionTemplate,
    '',
    '---',
    '',
    '# Поточний вміст файлу проекції (для consistency, не дублюй сліпо)',
    '',
    '```markdown',
    currentContent || '(файл порожній — створи з нуля на основі ADR)',
    '```',
    '',
    '---',
    '',
    `# ADR MLMaiL (${adrs.length} clean, повним body)`,
    '',
    adrSection,
    '',
    '---',
    '',
    '# Інструкція до відповіді',
    '',
    'Поверни рівно один JSON-обʼєкт без markdown-fence, без преамбули:',
    '```',
    '{ "content": "<повний markdown файлу>", "used_adrs": ["<slug>", ...] }',
    '```',
    '',
    `Файл, який ти генеруєш: docs/ci4/${name}.md. Зроби його повним, самодостатнім, готовим до коміту.`
  ].join('\n')
}

/**
 * Best-effort dump of a raw LLM response for debugging.
 * @param {string} rootDir Repository root directory.
 * @param {string} name Projection name.
 * @param {string} suffix Attempt suffix (e.g. `attempt1`).
 * @param {string} raw Raw LLM output to save.
 * @returns {Promise<void>} Resolves once the dump is attempted.
 */
async function saveDebug(rootDir, name, suffix, raw) {
  const debugDir = join(rootDir, 'docs/ci4/.regen-debug')
  try {
    await mkdir(debugDir, { recursive: true })
    const stamp = new Date().toISOString().replaceAll(/[.:]/g, '-')
    await writeFile(join(debugDir, `${name}-${stamp}-${suffix}.txt`), raw, 'utf8')
  } catch {
    // best-effort debug; ignore filesystem errors
  }
}

/**
 * Strip a trailing "Опрацьовано" mark block from an ADR body.
 * @param {string} body ADR body text.
 * @returns {string} Body without the trailing mark block.
 */
function stripExistingMark(body) {
  const idx = body.lastIndexOf('\n---\n\n**Опрацьовано**')
  if (idx === -1) {
    const altIdx = body.lastIndexOf('\n---\n**Опрацьовано**')
    if (altIdx === -1) return body
    return body.slice(0, altIdx).trimEnd()
  }
  return body.slice(0, idx).trimEnd()
}
