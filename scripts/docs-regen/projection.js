import { writeFile, mkdir } from 'node:fs/promises'
import { join } from 'node:path'
import { callLlm, parseLlmResponse } from './llm.js'

export async function regenerateProjection({
  name,
  adrs,
  currentContent,
  templates,
  model,
  rootDir,
}) {
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
        `Projection ${name}: LLM response unparseable after retry. ` +
          `First: ${error.message}. Second: ${retryError.message}. ` +
          `Raw outputs saved to docs/ci4/.regen-debug/`,
      )
    }
  }

  return {
    content: parsed.content,
    used_adrs: parsed.used_adrs,
    prompt_length: prompt.length,
    output_length: raw.length,
  }
}

function buildPrompt({ name, adrs, currentContent, templates }) {
  const globalRules = templates['_global.prompt.md']
  const projectionTemplate = templates[`${name}.prompt.md`]

  const adrSection = adrs
    .map((a) => `### ADR: ${a.slug}\n\n${stripExistingMark(a.body)}`)
    .join('\n\n')

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
    `Файл, який ти генеруєш: docs/ci4/${name}.md. Зроби його повним, самодостатнім, готовим до коміту.`,
  ].join('\n')
}

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

function stripExistingMark(body) {
  const idx = body.lastIndexOf('\n---\n\n**Опрацьовано**')
  if (idx === -1) {
    const altIdx = body.lastIndexOf('\n---\n**Опрацьовано**')
    if (altIdx === -1) return body
    return body.slice(0, altIdx).trimEnd()
  }
  return body.slice(0, idx).trimEnd()
}
