<script setup>
import LoginView from './views/Login.vue'
import TasksPanel from './components/TasksPanel.vue'
import { useAuthStore } from './services/auth-store.js'
import { useUpdater } from './composables/use-updater.js'
import { useTaskScan } from './composables/use-task-scan.js'
import { listTemplates } from './services/newsletter-template.js'

const auth = useAuthStore()
const taskScan = useTaskScan()
useUpdater()

// Trigger task scan whenever user becomes authenticated.
watch(
  () => auth.isAuthenticated.value,
  async (authed) => {
    if (!authed) return
    const templates = await listTemplates()
    taskScan.scan(templates)
  },
)
</script>

<template>
  <q-layout view="hHh lpR fFf">
    <q-page-container>
      <LoginView />
    </q-page-container>

    <TasksPanel
      v-if="auth.isAuthenticated.value"
      :tasks="taskScan.tasks.value"
      :is-scanning="taskScan.isScanning.value"
      :scanned-count="taskScan.scannedCount.value"
      :total-count="taskScan.totalCount.value"
      @remove-message="taskScan.removeMessage" />
  </q-layout>
</template>
