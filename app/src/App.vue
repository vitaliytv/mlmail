<script setup>
import { AgentDialog, AuditDialog } from '@7n/tauri-components/components'
import LoginView from './views/Login.vue'
import { useAuthStore } from './services/auth-store.js'
import { useAgent } from './composables/use-agent.js'
import { useUpdater } from './composables/use-updater.js'

const auth = useAuthStore()
const agent = useAgent()
useUpdater()
const agentOpen = ref(false)
const auditOpen = ref(false)
</script>

<template>
  <q-layout view="hHh lpR fFf">
    <q-page-container>
      <LoginView />
    </q-page-container>

    <q-footer v-if="auth.isAuthenticated.value" elevated class="bg-white text-dark">
      <q-toolbar class="justify-center q-gutter-sm">
        <q-btn
          @click="auth.loadRandomMessage()"
          color="primary"
          icon="sym_o_refresh"
          :loading="auth.isMessageLoading.value">
          Показати інший
        </q-btn>
        <q-btn @click="agentOpen = true" flat color="primary" icon="sym_o_smart_toy">Агент</q-btn>
        <q-btn @click="auditOpen = true" flat color="grey-8" icon="sym_o_history" title="Журнал запитів" />
        <q-btn @click="auth.logout()" flat color="grey-8" icon="sym_o_logout">Вийти</q-btn>
      </q-toolbar>
    </q-footer>

    <AgentDialog v-model="agentOpen" :agent="agent" />
    <AuditDialog v-model="auditOpen" :agent="agent" />
  </q-layout>
</template>
