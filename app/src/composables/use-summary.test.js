import { beforeEach, describe, expect, it, vi } from 'vitest'

const { chatMock, loadEnvMock } = vi.hoisted(() => ({ chatMock: vi.fn(), loadEnvMock: vi.fn() }))

vi.mock('../llm.js', () => ({ useLlm: () => ({ chat: chatMock, loadEnv: loadEnvMock }) }))

const { useSummary } = await import('./use-summary.js')

beforeEach(() => {
  chatMock.mockReset()
  loadEnvMock.mockReset()
  loadEnvMock.mockResolvedValue()
})

describe('useSummary', () => {
  it('shows an actionable queue-full error instead of hiding it behind a generic failure', async () => {
    chatMock.mockRejectedValue(new Error('generation queue is full (queue_full)'))
    const summary = useSummary()

    await expect(summary.summarize({ body: 'Лист для резюме' })).resolves.toBeNull()

    expect(summary.summaryError.value).toContain('черга генерації переповнена')
    expect(summary.summaryError.value).toContain('queue_full')
  })
})
