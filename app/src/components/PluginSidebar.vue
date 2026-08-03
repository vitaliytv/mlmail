<template>
  <!-- Plugin A2UI sidebar (M4–M5): validated nitra.core surface + draft action -->
  <q-card flat bordered class="plugin-sidebar column">
    <q-card-section class="row items-center q-pb-none">
      <div class="text-subtitle2">Плагін · sidebar</div>
      <q-space />
      <q-spinner v-if="loading" size="18px" color="primary" />
      <q-btn v-else @click="reload" flat dense round icon="sym_o_refresh" aria-label="Оновити панель плагіна" />
    </q-card-section>
    <q-separator />
    <q-card-section class="col">
      <div v-if="loadError" class="text-negative text-caption">{{ loadError }}</div>
      <A2uiSurface v-else-if="surface" :surface="surface" @action="onAction" @error="onRenderError" />
      <div v-else class="text-grey-6 text-caption">Немає поверхні плагіна.</div>
      <div v-if="lastAction" class="text-caption text-grey-6 q-mt-sm">Остання дія: {{ lastAction }}</div>
    </q-card-section>
  </q-card>
</template>

<script setup>
/**
 * Host slot for A2UI sidebar: loads a Rust-validated sample surface
 * (`plugin_a2ui_sample_sidebar`) and routes createDraft via
 * `plugin_sidebar_create_draft` (Wasm handle_action + audit).
 */
import { invoke } from '@tauri-apps/api/core'
import A2uiSurface from './A2uiSurface.vue'

const loading = ref(false)
const loadError = ref('')
const surface = ref(null)
const lastAction = ref('')

async function reload() {
  loading.value = true
  loadError.value = ''
  try {
    surface.value = await invoke('plugin_a2ui_sample_sidebar')
  } catch (error) {
    surface.value = null
    loadError.value = error?.message || String(error)
  } finally {
    loading.value = false
  }
}

function onAction(payload) {
  const name = payload?.action?.event?.name || 'action'
  lastAction.value = name
  if (name === 'createDraft') {
    runCreateDraft()
  }
}

async function runCreateDraft() {
  loading.value = true
  loadError.value = ''
  try {
    const r = await invoke('plugin_sidebar_create_draft')
    lastAction.value = `createDraft → ${r.draftId} (${r.auditResult})`
  } catch (error) {
    loadError.value = error?.message || String(error)
  } finally {
    loading.value = false
  }
}

function onRenderError(msg) {
  loadError.value = msg
}

onMounted(() => {
  reload()
})
</script>

<style scoped>
.plugin-sidebar {
  min-height: 160px;
  max-width: 100%;
}
</style>
