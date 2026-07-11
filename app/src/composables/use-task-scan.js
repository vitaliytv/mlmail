import { invoke } from '@tauri-apps/api/core'

/** Gmail search query that matches every message under the "Задача" label. */
const TASK_LABEL_QUERY = 'label:"Задача"'

/**
 * Build a Gmail search query from template from/subject patterns.
 * @param {{ from_pattern: string, subject_pattern: string }} t
 * @returns {string}
 */
function buildQuery(t) {
  const parts = []
  if (t.from_pattern) parts.push(`from:${t.from_pattern}`)
  if (t.subject_pattern) parts.push(`subject:"${t.subject_pattern}"`)
  return parts.join(' ')
}

const tasks = ref(/** @type {object[]} */ ([]))
const isScanning = ref(false)
const scannedCount = ref(0)
const totalCount = ref(0)

/** Shared singleton so any view can flag/refresh and see it reflected in the tasks panel. */
export function useTaskScan() {
  /** Reload the flat list of every message currently under the "Задача" label. */
  async function refresh() {
    try {
      tasks.value = await invoke('gmail_search', { q: TASK_LABEL_QUERY })
    } catch {
      tasks.value = []
    }
  }

  /**
   * Apply the "Задача" label to every message matching each task template
   * (promoting new matches), then reload the flat task list.
   * @param {import('../services/newsletter-template.js').NewsletterTemplate[]} templates
   */
  async function scan(templates) {
    const taskTemplates = templates.filter(t => t.type === 'task')

    isScanning.value = true
    scannedCount.value = 0
    totalCount.value = taskTemplates.length

    await Promise.all(
      taskTemplates.map(async template => {
        const q = buildQuery(template)
        if (!q) return
        try {
          const messages = await invoke('gmail_search', { q })
          await Promise.all(messages.map(m => invoke('gmail_flag_task', { id: m.id })))
        } catch {
          /* ignore per-template failures */
        } finally {
          scannedCount.value++
        }
      })
    )

    await refresh()
    isScanning.value = false
  }

  /**
   * Flag one message as a task and refresh the list.
   * @param {string} id Gmail message id
   */
  async function flagMessage(id) {
    await invoke('gmail_flag_task', { id })
    await refresh()
  }

  /**
   * Mark a task done (unflag) and drop it from the local list.
   * @param {string} id Gmail message id
   */
  async function unflagMessage(id) {
    await invoke('gmail_unflag_task', { id })
    tasks.value = tasks.value.filter(m => m.id !== id)
  }

  const totalTasks = computed(() => tasks.value.length)

  return { tasks, isScanning, scannedCount, totalCount, totalTasks, scan, refresh, flagMessage, unflagMessage }
}
