export function parseLlmResponse(raw) {
  let text = raw.trim()
  const fenceMatch = text.match(/^```(?:json)?\s*\n([\s\S]*?)\n```\s*$/m)
  if (fenceMatch) {
    text = fenceMatch[1].trim()
  }
  if (!text.startsWith('{')) {
    const objMatch = text.match(/\{[\s\S]*\}/)
    if (objMatch) text = objMatch[0]
  }

  let parsed
  try {
    parsed = JSON.parse(text)
  } catch (error) {
    throw new Error(`Failed to parse LLM response as JSON: ${error.message}`)
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
    args: (model) => ['-p', '--model', model],
    defaultModel: 'sonnet',
  },
  {
    name: 'cursor-agent',
    args: (model) => ['-p', '--mode', 'ask', '--output-format', 'text', '--model', model],
    defaultModel: 'claude-4.6-sonnet-medium',
  },
]

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
        stderr: 'pipe',
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
