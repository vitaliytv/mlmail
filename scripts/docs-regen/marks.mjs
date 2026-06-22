const MARK_HEADER = '**Опрацьовано**'

/**
 * Detect sentinel mark block at the end of an ADR body.
 * A mark is: last `---` (horizontal rule on its own line),
 * followed (after optional blank line) by a paragraph that starts with `**Опрацьовано**`.
 * @param {string} rawContent Raw ADR file content.
 * @returns {boolean} True if the mark block is present at the end.
 */
export function hasMark(rawContent) {
  return findMarkStart(rawContent) !== -1
}

/**
 * Return ADR body with the trailing mark block removed (if any).
 * @param {string} rawContent Raw ADR file content.
 * @returns {string} Body without the mark block, ending in a single `\n`.
 */
export function stripMark(rawContent) {
  const idx = findMarkStart(rawContent)
  if (idx === -1) return rawContent
  let body = rawContent.slice(0, idx)
  while (body.endsWith('\n')) body = body.slice(0, -1)
  return body + '\n'
}

/**
 * Build the textual mark block (without the leading `---` separator).
 * @param {string} date Date in ISO `YYYY-MM-DD` format.
 * @param {string[]} projections Projection names this ADR appears in.
 * @returns {string} Mark paragraph text.
 */
export function formatMark(date, projections) {
  if (projections.length === 0) {
    return `${MARK_HEADER} ${date}. Проекції: жодної.`
  }
  const lines = projections.map(p => `- [${p}](../ci4/${p}.md)`)
  return `${MARK_HEADER} ${date}. Проекції:\n${lines.join('\n')}`
}

/**
 * Apply (insert or replace) the mark block at the end of the ADR.
 * @param {string} rawContent Raw ADR file content.
 * @param {string} date Date in ISO `YYYY-MM-DD` format.
 * @param {string[]} projections Projection names this ADR appears in.
 * @returns {string} Updated ADR content with fresh mark at the end.
 */
export function applyMark(rawContent, date, projections) {
  const stripped = stripMark(rawContent)
  const body = stripped.endsWith('\n') ? stripped : stripped + '\n'
  return body + '\n---\n\n' + formatMark(date, projections) + '\n'
}

/**
 * Find the offset where the trailing mark block starts.
 * @param {string} rawContent Raw ADR file content.
 * @returns {number} Offset of the mark's `---` separator, or `-1` if absent.
 */
function findMarkStart(rawContent) {
  const lines = rawContent.split('\n')
  for (let i = lines.length - 1; i >= 0; i--) {
    if (lines[i].trim() !== '---') continue
    let j = i + 1
    while (j < lines.length && lines[j].trim() === '') j++
    if (j < lines.length && lines[j].startsWith(MARK_HEADER)) {
      let offset = 0
      for (let k = 0; k < i; k++) offset += lines[k].length + 1
      while (offset > 0 && rawContent[offset - 1] === '\n') offset--
      return offset
    }
  }
  return -1
}
