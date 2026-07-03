// Pure helper for the Ukrainian email-summary feature: turn a message into the
// user-role prompt fed to the local LLM. Side-effect free so it's unit-testable;
// the LLM call (I/O) lives in composables/use-summary.js.

/**
 * System prompt instructing the local model to summarize an email in Ukrainian.
 */
export const SUMMARY_SYSTEM = [
  'Ти стисло переказуєш зміст email українською мовою.',
  'Дай коротке резюме (2–5 речень або кілька пунктів): хто відправник, що сталося',
  'та що від користувача очікують або пропонують.',
  'Пиши лише сам переказ українською — без вступів на кшталт «Ось резюме».',
].join(' ')

export const TRANSLATE_SYSTEM = [
  'Ти перекладач електронних листів.',
  'У вхідному тексті може бути CSS-стилі, HTML-атрибути або технічний код — повністю їх ігноруй.',
  'Переклади лише читабельний людський текст листа на українську мову.',
  'Зберігай структуру: абзаци, списки, заголовки.',
  'Пиши лише переклад — без коментарів, пояснень та вступів.',
].join(' ')

export const TRANSLATE_BATCH_SYSTEM = [
  'Ти перекладач. Тобі надається JSON-масив рядків англійського тексту з email.',
  'Переклади кожен рядок на українську. Повертай ТІЛЬКИ JSON-масив рядків — без коментарів та markdown.',
  'Якщо рядок порожній або це не текст (URL, число, пробіл) — повертай його без змін.',
  'Кількість елементів у відповіді має бути РІВНО такою самою, як у вхідному масиві.',
].join(' ')

/**
 * Build the user-role prompt for summarizing a message.
 * @param {{ from?: string, subject?: string, body?: string }} message the email
 * @returns {string} the prompt text (sender, subject, then body)
 */
export function buildSummaryPrompt(message) {
  const from = (message?.from ?? '').trim()
  const subject = (message?.subject ?? '').trim()
  const body = (message?.body ?? '').trim()
  const head = [from && `Від: ${from}`, subject && `Тема: ${subject}`].filter(Boolean).join('\n')
  return head ? `${head}\n\n${body}` : body
}

/**
 * Strip CSS blocks that end up in plain-text extraction of HTML emails,
 * then build the translation prompt.
 * @param {{ from?: string, subject?: string, body?: string }} message
 * @returns {string}
 */
export function buildTranslatePrompt(message) {
  const from = (message?.from ?? '').trim()
  const subject = (message?.subject ?? '').trim()
  // Remove anything that looks like a CSS rule block: selector { ... }
  const body = (message?.body ?? '')
    .replaceAll(/[^{}]*\{[^{}]*\}/g, '')
    .replaceAll(/\n{3,}/g, '\n\n')
    .trim()
  const head = [from && `Від: ${from}`, subject && `Тема: ${subject}`].filter(Boolean).join('\n')
  return head ? `${head}\n\n${body}` : body
}

// Tags whose text content we never want to translate.
const SKIP_TAGS = new Set(['STYLE', 'SCRIPT', 'NOSCRIPT', 'HEAD', 'META', 'LINK', 'TITLE'])

/**
 * Walk a DOM node tree and collect all non-empty text nodes along with
 * a stable placeholder key. Returns `{ nodes, texts }` where `texts` is
 * the array to send to the LLM and `nodes` are the actual Text DOM nodes.
 * @param {Node} root
 * @returns {{ nodes: Text[], texts: string[] }}
 */
export function extractTextNodes(root) {
  const nodes = []
  const texts = []
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode(node) {
      const parent = node.parentElement
      if (!parent) return NodeFilter.FILTER_REJECT
      if (SKIP_TAGS.has(parent.tagName)) return NodeFilter.FILTER_REJECT
      const text = node.textContent ?? ''
      // Skip whitespace-only nodes
      if (!text.trim()) return NodeFilter.FILTER_REJECT
      return NodeFilter.FILTER_ACCEPT
    },
  })
  let node
  while ((node = walker.nextNode())) {
    nodes.push(node)
    texts.push(node.textContent)
  }
  return { nodes, texts }
}

/**
 * Parse an HTML string, translate all text nodes via `translateBatch`,
 * and return the translated HTML string.
 * @param {string} html raw HTML of the email
 * @param {(texts: string[]) => Promise<string[]>} translateBatch
 * @returns {Promise<string>}
 */
export async function translateHtmlEmail(html, translateBatch) {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')

  const { nodes, texts } = extractTextNodes(doc.body)
  if (!texts.length) return html

  const translated = await translateBatch(texts)

  // Apply translations back; fall back to original text on length mismatch.
  nodes.forEach((node, i) => {
    const replacement = translated[i]
    if (replacement && typeof replacement === 'string') {
      node.textContent = replacement
    }
  })

  return doc.documentElement.outerHTML
}
