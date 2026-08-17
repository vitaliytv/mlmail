<template>
  <q-dialog @update:model-value="onToggle" :model-value="modelValue" maximized>
    <q-card>
      <q-bar>
        <span class="text-weight-medium">Плагіни</span>
        <q-space />
        <q-btn v-close-popup flat round dense icon="sym_o_close" />
      </q-bar>

      <q-card-section class="column q-gutter-sm">
        <div class="row q-gutter-sm items-center">
          <q-btn
            @click="pickAndInstall"
            color="primary"
            no-caps
            icon="sym_o_folder_open"
            label="Встановити .n-plugin"
            :loading="busy" />
          <q-btn @click="reload" flat no-caps icon="sym_o_refresh" label="Оновити" :disable="busy" />
        </div>
        <div class="text-caption text-grey-7">
          Можна встановити тільки typed Draft Helper WebAssembly Component. ZIP/core-Wasm пакети не підтримуються.
        </div>
        <div v-if="error" class="text-negative text-caption">{{ error }}</div>
      </q-card-section>

      <q-card-section>
        <div v-if="!plugins.length" class="text-grey-6">Плагінів ще немає.</div>
        <q-list v-else bordered separator rounded>
          <q-item v-for="plugin in plugins" :key="plugin.release.digest">
            <q-item-section>
              <q-item-label class="row items-center q-gutter-xs">
                <span>{{ plugin.release.package }}</span>
                <q-badge color="grey-7" :label="plugin.release.version" />
                <q-badge v-if="!plugin.enabled" color="orange" label="Вимкнено" />
              </q-item-label>
              <q-item-label caption>{{ plugin.release.digest }}</q-item-label>
              <q-item-label caption>Triggers: {{ plugin.triggers.join(', ') }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <div class="row q-gutter-xs">
                <q-btn
                  v-if="canCreateDraft(plugin)"
                  @click="createDraft"
                  color="primary"
                  dense
                  no-caps
                  label="Create draft"
                  :loading="busy" />
                <q-btn
                  @click="toggleDisabled(plugin)"
                  flat
                  dense
                  no-caps
                  :label="plugin.enabled ? 'Вимкнути' : 'Увімкнути'" />
                <q-btn
                  @click="uninstall(plugin)"
                  flat
                  dense
                  round
                  icon="sym_o_delete"
                  color="negative"
                  aria-label="Видалити плагін" />
              </div>
            </q-item-section>
          </q-item>
        </q-list>
        <div v-if="lastDraft" class="text-positive text-caption q-mt-md">{{ lastDraft }}</div>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup>
/**
 * Component-only Plugin Manager for installing and invoking the typed Draft Helper demo.
 */
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const DRAFT_HELPER_TRIGGER = 'nitra:gmail/draft-helper@0.1.0'

const props = defineProps({
  modelValue: { type: Boolean, default: false }
})
const emit = defineEmits(['update:modelValue'])

const plugins = ref([])
const busy = ref(false)
const error = ref('')
const lastDraft = ref('')

/** Синхронізує відкриття діалогу та одразу завантажує актуальний список. */
function onToggle(value) {
  emit('update:modelValue', value)
  if (value) reload()
}

/** Завантажує локальну product projection встановлених Components. */
async function reload() {
  error.value = ''
  try {
    plugins.value = await invoke('plugin_manager_list')
  } catch (error) {
    error.value = error?.message || String(error)
  }
}

/** Повертає, чи доступний цьому увімкненому Component Draft Helper trigger. */
function canCreateDraft(plugin) {
  return plugin.enabled && plugin.triggers.includes(DRAFT_HELPER_TRIGGER)
}

/** Відкриває нативний picker і передає вибраний Component у typed installer. */
async function pickAndInstall() {
  busy.value = true
  error.value = ''
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'n-plugin Component', extensions: ['n-plugin'] }]
    })
    if (path) {
      await invoke('plugin_manager_install', { path })
      await reload()
    }
  } catch (error) {
    error.value = error?.message || String(error)
  } finally {
    busy.value = false
  }
}

/** Змінює явний user enablement без зміни immutable Component bytes. */
async function toggleDisabled(plugin) {
  busy.value = true
  error.value = ''
  try {
    await invoke('plugin_manager_set_disabled', {
      package: plugin.release.package,
      disabled: plugin.enabled
    })
    await reload()
  } catch (error) {
    error.value = error?.message || String(error)
  } finally {
    busy.value = false
  }
}

/** Вимикає Component generation та прибирає його з product projection. */
async function uninstall(plugin) {
  busy.value = true
  error.value = ''
  try {
    await invoke('plugin_manager_uninstall', { package: plugin.release.package })
    await reload()
  } catch (error) {
    error.value = error?.message || String(error)
  } finally {
    busy.value = false
  }
}

/** Викликає typed Draft Helper trigger для поточного авторизованого акаунта. */
async function createDraft() {
  busy.value = true
  error.value = ''
  lastDraft.value = ''
  try {
    const result = await invoke('plugin_draft_helper_create')
    lastDraft.value = `Чернетку ${result.draftId} створено через ${result.release.package}.`
  } catch (error) {
    error.value = error?.message || String(error)
  } finally {
    busy.value = false
  }
}

watch(
  () => props.modelValue,
  value => {
    if (value) reload()
  }
)
</script>
