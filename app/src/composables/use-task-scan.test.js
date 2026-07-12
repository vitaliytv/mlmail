import { describe, it, expect, vi, beforeEach } from 'vitest'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { useTaskScan } = await import('./use-task-scan.js')

beforeEach(() => {
  invokeMock.mockReset()
})

describe('useTaskScan.refresh', () => {
  it('searches the "Задача" label while excluding trashed messages', async () => {
    invokeMock.mockResolvedValue([])
    const taskScan = useTaskScan()
    await taskScan.refresh()
    expect(invokeMock).toHaveBeenCalledWith('gmail_search', { q: 'label:"Задача" -in:trash' })
  })

  it('populates tasks from the search result', async () => {
    const messages = [{ id: 'm1', from: 'a@e', subject: 's', date: 'd' }]
    invokeMock.mockResolvedValue(messages)
    const taskScan = useTaskScan()
    await taskScan.refresh()
    expect(taskScan.tasks.value).toEqual(messages)
  })

  it('clears tasks when the search fails', async () => {
    invokeMock.mockRejectedValue(new Error('boom'))
    const taskScan = useTaskScan()
    await taskScan.refresh()
    expect(taskScan.tasks.value).toEqual([])
  })
})
