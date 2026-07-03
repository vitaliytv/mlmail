import { invoke } from '@tauri-apps/api/core'

/**
 * @typedef {{ template: import('../services/newsletter-template.js').NewsletterTemplate, messages: object[] }} TaskGroup
 */

/**
 * Build a Gmail search query from template patterns.
 * @param {{ from_pattern: string, subject_pattern: string }} t
 * @returns {string}
 */
function buildQuery(t) {
  const parts = []
  if (t.from_pattern) parts.push(`from:${t.from_pattern}`)
  if (t.subject_pattern) parts.push(`subject:"${t.subject_pattern}"`)
  return parts.join(' ')
}

export function useTaskScan() {
  const tasks = ref(/** @type {TaskGroup[]} */ ([]))
  const isScanning = ref(false)
  const scannedCount = ref(0)
  const totalCount = ref(0)

  /**
   * Scan inbox for all task templates. Runs asynchronously — updates tasks reactively.
   * @param {import('../services/newsletter-template.js').NewsletterTemplate[]} templates
   */
  async function scan(templates) {
    const taskTemplates = templates.filter(t => t.type === 'task')
    if (!taskTemplates.length) return

    isScanning.value = true
    scannedCount.value = 0
    totalCount.value = taskTemplates.length
    tasks.value = []

    await Promise.all(taskTemplates.map(async (template) => {
      const q = buildQuery(template)
      if (!q) return
      try {
        const messages = await invoke('gmail_search', { q })
        if (messages.length > 0) {
          tasks.value = [...tasks.value, { template, messages }]
        }
      }
      catch { /* ignore per-template failures */ }
      finally {
        scannedCount.value++
      }
    }))

    isScanning.value = false
  }

  /** Remove a specific message from the task list after trashing. */
  function removeMessage(templateId, messageId) {
    tasks.value = tasks.value
      .map(g => g.template.id === templateId
        ? { ...g, messages: g.messages.filter(m => m.id !== messageId) }
        : g)
      .filter(g => g.messages.length > 0)
  }

  const totalTasks = computed(() => tasks.value.reduce((s, g) => s + g.messages.length, 0))

  return { tasks, isScanning, scannedCount, totalCount, totalTasks, scan, removeMessage }
}
