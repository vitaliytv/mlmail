<template>
  <q-dialog :model-value="modelValue" @update:model-value="onToggle" maximized>
    <q-card>
      <q-bar>
        <span class="text-weight-medium">Плагіни</span>
        <q-space />
        <q-btn v-close-popup flat round dense icon="sym_o_close" />
      </q-bar>

      <q-card-section class="column q-gutter-sm">
        <div class="row q-gutter-sm items-center">
          <q-btn
            @click="installSample"
            color="primary"
            no-caps
            icon="sym_o_download"
            label="Встановити signed sample"
            :loading="busy" />
          <q-btn @click="reload" flat no-caps icon="sym_o_refresh" label="Оновити" :disable="busy" />
        </div>
        <div class="row q-gutter-sm items-center">
          <q-input v-model="packagePath" class="col" dense outlined label="Шлях до .n-plugin" :disable="busy" />
          <q-btn
            @click="installFromPath"
            color="secondary"
            no-caps
            icon="sym_o_folder_open"
            label="Встановити з файлу"
            :loading="busy"
            :disable="!packagePath.trim()" />
        </div>
        <div v-if="error" class="text-negative text-caption">{{ error }}</div>
      </q-card-section>

      <q-card-section>
        <div v-if="!plugins.length" class="text-grey-6">Плагінів ще немає.</div>
        <q-list v-else bordered separator rounded>
          <q-item v-for="p in plugins" :key="p.id">
            <q-item-section>
              <q-item-label class="row items-center q-gutter-xs">
                <span>{{ p.name }}</span>
                <q-badge color="grey-7" :label="p.version" />
                <q-badge v-if="p.disabled" color="orange" label="Вимкнено" />
              </q-item-label>
              <q-item-label caption>
                {{ p.publisher }} · {{ p.id }}
                <span v-if="p.fingerprint"> · fp {{ p.fingerprint }}</span>
              </q-item-label>
              <q-item-label caption>
                Grants: {{ p.granted?.length || 0 }} · caps:
                {{ (p.capabilities || []).map(c => c.name).join(', ') }}
              </q-item-label>
            </q-item-section>
            <q-item-section side>
              <div class="row q-gutter-xs">
                <q-btn @click="toggleDisabled(p)" flat dense no-caps :label="p.disabled ? 'Увімкнути' : 'Вимкнути'" />
                <q-btn
                  @click="uninstall(p)"
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
      </q-card-section>

      <PluginConsentDialog v-model="consentOpen" :preview="consentPreview" @accept="confirmInstall" />
    </q-card>
  </q-dialog>
</template>

<script setup>
/**
 * Plugin Manager: list / signed sample / path install / disable / uninstall purge.
 */
import { invoke } from '@tauri-apps/api/core'
import PluginConsentDialog from './PluginConsentDialog.vue'

const props = defineProps({
  modelValue: { type: Boolean, default: false }
})
const emit = defineEmits(['update:modelValue'])

const plugins = ref([])
const busy = ref(false)
const error = ref('')
const consentOpen = ref(false)
const consentPreview = ref(null)
const pendingPath = ref('')

function onToggle(v) {
  emit('update:modelValue', v)
  if (v) reload()
}

async function reload() {
  error.value = ''
  try {
    plugins.value = await invoke('plugin_manager_list')
  } catch (err) {
    error.value = err?.message || String(err)
  }
}

async function installSample() {
  busy.value = true
  error.value = ''
  try {
    await invoke('plugin_manager_install_sample')
    await reload()
  } catch (err) {
    error.value = err?.message || String(err)
  } finally {
    busy.value = false
  }
}

async function toggleDisabled(p) {
  busy.value = true
  error.value = ''
  try {
    await invoke('plugin_manager_set_disabled', {
      pluginId: p.id,
      disabled: !p.disabled
    })
    await reload()
  } catch (err) {
    error.value = err?.message || String(err)
  } finally {
    busy.value = false
  }
}

async function uninstall(p) {
  busy.value = true
  error.value = ''
  try {
    await invoke('plugin_manager_uninstall', { pluginId: p.id })
    await reload()
  } catch (err) {
    error.value = err?.message || String(err)
  } finally {
    busy.value = false
  }
}

async function previewPath(path) {
  pendingPath.value = path
  consentPreview.value = await invoke('plugin_manager_preview_install', { path })
  consentOpen.value = true
}

async function installFromPath() {
  const path = packagePath.value.trim()
  if (!path) return
  busy.value = true
  error.value = ''
  try {
    await previewPath(path)
  } catch (err) {
    error.value = err?.message || String(err)
  } finally {
    busy.value = false
  }
}

async function confirmInstall() {
  if (!pendingPath.value || !consentPreview.value) return
  busy.value = true
  error.value = ''
  try {
    const caps = consentPreview.value.manifest.capabilities || []
    const grants = caps.flatMap(c => {
      const kinds = c.resourceKinds || c.resource_kinds || []
      return kinds.map(kind => ({
        capability: c.name,
        resourceKind: kind,
        resourceId: kind === 'message' ? 'msg_1' : kind === 'account' ? 'acct_1' : null
      }))
    })
    await invoke('plugin_manager_install', {
      path: pendingPath.value,
      grants,
      tofuAccept: true
    })
    consentOpen.value = false
    await reload()
  } catch (err) {
    error.value = err?.message || String(err)
  } finally {
    busy.value = false
  }
}

watch(
  () => props.modelValue,
  v => {
    if (v) reload()
  }
)
</script>
