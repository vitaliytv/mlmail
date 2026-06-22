/**
 * Parse a raw LLM response into the expected projection result shape.
 * @param {string} raw Raw text returned by the LLM CLI.
 * @returns {{content: string, used_adrs: string[]}} Parsed projection payload.
 */
export function parseLlmResponse(raw) {
  let text = raw.trim()
  // Unwrap a ```json … ``` (or bare ```) fenced block without regex backtracking.
  if (text.startsWith('```')) {
    const firstNewline = text.indexOf('\n')
    const closingFence = text.lastIndexOf('\n```')
    if (firstNewline !== -1 && closingFence > firstNewline) {
      text = text.slice(firstNewline + 1, closingFence).trim()
    }
  }
  // Drop any surrounding prose by keeping the outermost { … } span.
  if (!text.startsWith('{')) {
    const start = text.indexOf('{')
    const end = text.lastIndexOf('}')
    if (start !== -1 && end > start) {
      text = text.slice(start, end + 1)
    }
  }

  let parsed
  try {
    parsed = JSON.parse(text)
  } catch (error) {
    throw new Error(`Failed to parse LLM response as JSON: ${error.message}`, { cause: error })
  }

  if (typeof parsed.content !== 'string') {
    throw new TypeError('LLM response missing required field "content" (string)')
  }
  if (!Array.isArray(parsed.used_adrs)) {
    throw new TypeError('LLM response missing required field "used_adrs" (array)')
  }
  for (const item of parsed.used_adrs) {
    if (typeof item !== 'string') {
      throw new TypeError('LLM response field "used_adrs" must contain only strings')
    }
  }
  return { content: parsed.content, used_adrs: parsed.used_adrs }
}

const CLI_CANDIDATES = [
  {
    name: 'claude',
    args: model => ['-p', '--model', model],
    defaultModel: 'sonnet'
  },
  {
    name: 'cursor-agent',
    args: model => ['-p', '--mode', 'ask', '--output-format', 'text', '--model', model],
    defaultModel: 'claude-4.6-sonnet-medium'
  }
]

/**
 * Call the first available LLM CLI with the given prompt.
 * @param {string} prompt Prompt text sent to the CLI via stdin.
 * @param {{model?: string}} [opts] Call options.
 * @returns {Promise<string>} Raw stdout from the LLM CLI.
 */
export async function callLlm(prompt, opts = {}) {
  const errors = []
  for (const candidate of CLI_CANDIDATES) {
    const cli = candidate.name
    const model = opts.model ?? candidate.defaultModel
    const args = candidate.args(model)
    try {
      const proc = Bun.spawn([cli, ...args], {
        stdin: 'pipe',
        stdout: 'pipe',
        stderr: 'pipe'
      })
      proc.stdin.write(prompt)
      await proc.stdin.end()
      const stdout = await new Response(proc.stdout).text()
      const exit = await proc.exited
      if (exit !== 0) {
        const stderr = await new Response(proc.stderr).text()
        errors.push(`${cli} exited ${exit}: ${stderr.trim()}`)
        continue
      }
      return stdout
    } catch (error) {
      if (error.code === 'ENOENT') {
        errors.push(`${cli} not found in PATH`)
        continue
      }
      throw error
    }
  }
  throw new Error('No LLM CLI available:\n' + errors.join('\n'))
}
