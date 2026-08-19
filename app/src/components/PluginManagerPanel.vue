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
            @click="pickAndPreflight"
            color="primary"
            no-caps
            icon="sym_o_folder_open"
            label="Встановити .n-plugin"
            :loading="installBusy" />
          <q-btn @click="reload()" flat no-caps icon="sym_o_refresh" label="Оновити" :disable="installBusy" />
        </div>
        <div class="text-caption text-grey-7">
          Менеджер приймає typed WebAssembly Components із contracts, які підтримує поточна версія mlmail. ZIP,
          core-Wasm і невідомі host interfaces не активуються.
        </div>
        <div v-if="error" class="text-negative text-caption">{{ error }}</div>
      </q-card-section>

      <q-card-section v-if="installPreview" class="q-pt-none">
        <q-card flat bordered>
          <q-card-section class="column q-gutter-xs">
            <div class="text-subtitle1">Перевірка перед встановленням</div>
            <div>{{ installPreview.release.package }} @ {{ installPreview.release.version }}</div>
            <div class="text-caption text-grey-7">{{ installPreview.release.digest }}</div>
            <div v-if="!installPreview.compatible" class="text-negative">
              {{ installPreview.reason }}
            </div>
            <div v-if="installPreview.actions.length" class="text-caption">
              Дії: {{ installPreview.actions.map(action => action.label).join(', ') }}
            </div>
            <div v-if="installPreview.dependencies.length" class="text-caption">
              Залежності:
              {{
                installPreview.dependencies
                  .map(dependency => `${dependency.package} ${dependency.requirement}`)
                  .join(', ')
              }}
            </div>
            <div v-if="!installPreview.requiredCapabilities.length" class="text-positive">
              Додаткові дозволи не потрібні.
            </div>
            <q-checkbox
              v-for="requirement in installPreview.requiredCapabilities"
              :key="requirement.requirementId"
              v-model="grantDecisions[requirement.requirementId]"
              :label="`${requirement.capability} · ${requirement.accountId || 'application'}`" />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn @click="cancelInstall" flat no-caps label="Скасувати" />
            <q-btn
              @click="confirmInstall"
              color="primary"
              no-caps
              label="Підтвердити встановлення"
              :loading="installBusy"
              :disable="!canConfirmInstall" />
          </q-card-actions>
        </q-card>
      </q-card-section>

      <q-card-section>
        <div v-if="!pluginRows.length" class="text-grey-6">Плагінів ще немає.</div>
        <q-list v-else bordered separator rounded>
          <q-item v-for="plugin in pluginRows" :key="plugin.release.digest" data-plugin-row>
            <q-item-section>
              <q-item-label class="row items-center q-gutter-xs">
                <span>{{ plugin.release.package }}</span>
                <q-badge color="grey-7" :label="plugin.release.version" />
                <q-badge v-if="!plugin.enabled" color="orange" label="Вимкнено" />
              </q-item-label>
              <q-item-label caption>{{ plugin.release.digest }}</q-item-label>
              <q-item-label caption>
                Lifecycle: {{ plugin.lifecycle.effectiveState }} · {{ plugin.lifecycle.desiredState }}
              </q-item-label>
              <q-item-label caption>Generation: {{ plugin.activationGeneration }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <div class="row q-gutter-xs">
                <template v-for="action in plugin.actions" :key="`${plugin.release.digest}:${action.kind}`">
                  <q-btn
                    v-if="action.kind === 'draft-helper-create'"
                    @click="createDraft(plugin)"
                    color="primary"
                    dense
                    no-caps
                    :label="action.label"
                    :loading="plugin.itemBusy"
                    :disable="!plugin.enabled" />
                  <q-btn
                    v-if="action.kind === 'booking-finder-find'"
                    @click="findBookings(plugin)"
                    color="primary"
                    dense
                    no-caps
                    :label="action.label"
                    :loading="plugin.itemBusy"
                    :disable="!plugin.enabled" />
                </template>
                <q-btn
                  @click="toggleDisabled(plugin)"
                  flat
                  dense
                  no-caps
                  :label="plugin.enabled ? 'Вимкнути' : 'Увімкнути'"
                  :loading="plugin.itemBusy" />
                <q-btn
                  @click="uninstall(plugin)"
                  flat
                  dense
                  no-caps
                  label="Видалити"
                  icon="sym_o_delete"
                  color="negative"
                  :loading="plugin.itemBusy" />
              </div>
            </q-item-section>
          </q-item>
        </q-list>
        <div v-if="lastResult" class="text-positive text-caption q-mt-md">{{ lastResult }}</div>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup>
/**
 * General-purpose Plugin Manager that previews typed Components, collects opaque consent decisions,
 * and invokes only product-owned typed actions against an exact backend-projected release.
 */
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps({
  modelValue: { type: Boolean, default: false }
})
const emit = defineEmits(['update:modelValue'])

const plugins = ref([])
const installPreview = ref(null)
const selectedPath = ref('')
const grantDecisions = ref({})
const installBusy = ref(false)
const itemLoading = ref({})
const error = ref('')
const lastResult = ref('')

const pluginRows = computed(() =>
  plugins.value.map(plugin => ({
    ...plugin,
    itemBusy: Boolean(itemLoading.value[plugin.release.digest])
  }))
)
const canConfirmInstall = computed(
  () =>
    installPreview.value?.compatible === true &&
    installPreview.value.requiredCapabilities.every(
      requirement => grantDecisions.value[requirement.requirementId] === true
    )
)

/**
 * Синхронізує modelValue; список завантажує єдиний watcher.
 * @param {boolean} value нове значення видимості
 */
function onToggle(value) {
  emit('update:modelValue', value)
}

/**
 * Нормалізує невідоме Tauri rejection для UI.
 * @param {unknown} error отримана помилка
 * @returns {string} повідомлення для користувача
 */
function errorMessage(error) {
  return error?.message || String(error)
}

/**
 * Читає committed projection без мутації поточного operation error.
 * @returns {Promise<string>} порожній рядок або list error
 */
async function loadPlugins() {
  try {
    plugins.value = await invoke('plugin_manager_list')
    return ''
  } catch (error) {
    return errorMessage(error)
  }
}

/** Завантажує committed product projection і показує list error. */
async function reload() {
  const listError = await loadPlugins()
  if (listError) error.value = listError
}

/** Очищає лише локальний installation candidate і consent choices. */
function clearInstallPreview() {
  installPreview.value = null
  selectedPath.value = ''
  grantDecisions.value = {}
}

/** Відкриває native picker і показує compatibility/consent preview без activation writes. */
async function pickAndPreflight() {
  installBusy.value = true
  error.value = ''
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'n-plugin Component', extensions: ['n-plugin'] }]
    })
    if (!path) return
    const preview = await invoke('plugin_manager_preflight', { path })
    selectedPath.value = path
    installPreview.value = preview
    grantDecisions.value = Object.fromEntries(
      preview.requiredCapabilities.map(requirement => [requirement.requirementId, false])
    )
  } catch (error) {
    error.value = errorMessage(error)
    clearInstallPreview()
  } finally {
    installBusy.value = false
  }
}

/** Скасовує candidate preview без виклику activation command. */
function cancelInstall() {
  clearInstallPreview()
}

/** Підтверджує exact preview, передаючи backend лише opaque requirement decisions. */
async function confirmInstall() {
  if (!canConfirmInstall.value) return
  installBusy.value = true
  error.value = ''
  try {
    const preview = installPreview.value
    await invoke('plugin_manager_confirm_install', {
      confirmation: {
        path: selectedPath.value,
        previewId: preview.previewId,
        expectedRelease: preview.release,
        grants: preview.requiredCapabilities.map(requirement => ({
          requirementId: requirement.requirementId,
          allow: grantDecisions.value[requirement.requirementId] === true
        }))
      }
    })
    clearInstallPreview()
    error.value = await loadPlugins()
  } catch (error) {
    error.value = errorMessage(error)
  } finally {
    installBusy.value = false
  }
}

/**
 * Серіалізує loading/error UI одного exact plugin item.
 * @param {object} plugin backend-projected plugin item
 * @param {() => Promise<void>} operation typed operation for this exact release
 * @param {{ reloadAfter?: boolean }} options post-operation behavior
 * @returns {Promise<void>} завершення operation та optional reload
 */
async function runItemOperation(plugin, operation, { reloadAfter = false } = {}) {
  const digest = plugin.release.digest
  itemLoading.value = { ...itemLoading.value, [digest]: true }
  error.value = ''
  let operationError = ''
  try {
    await operation()
  } catch (error) {
    operationError = errorMessage(error)
  }
  const reloadError = reloadAfter ? await loadPlugins() : ''
  error.value = operationError || reloadError
  itemLoading.value = { ...itemLoading.value, [digest]: false }
}

/**
 * Змінює manual enablement тільки для exact backend release item.
 * @param {object} plugin backend-projected plugin item
 */
async function toggleDisabled(plugin) {
  await runItemOperation(
    plugin,
    () =>
      invoke('plugin_manager_set_disabled', {
        target: plugin.release,
        disabled: plugin.enabled
      }),
    { reloadAfter: true }
  )
}

/**
 * Видаляє тільки exact backend release item і потім оновлює projection.
 * @param {object} plugin backend-projected plugin item
 */
async function uninstall(plugin) {
  await runItemOperation(plugin, () => invoke('plugin_manager_uninstall', { target: plugin.release }), {
    reloadAfter: true
  })
}

/**
 * Викликає typed Draft Helper command з exact ReleaseIdentity вибраного item.
 * @param {object} plugin backend-projected plugin item
 */
async function createDraft(plugin) {
  lastResult.value = ''
  await runItemOperation(plugin, async () => {
    const result = await invoke('plugin_draft_helper_create', { target: plugin.release })
    lastResult.value = `Чернетку ${result.draftId} створено через ${result.release.package}.`
  })
}

/**
 * Викликає typed Booking Finder command з exact ReleaseIdentity вибраного item.
 * @param {object} plugin backend-projected plugin item
 */
async function findBookings(plugin) {
  lastResult.value = ''
  await runItemOperation(plugin, async () => {
    const result = await invoke('plugin_booking_finder_find', { target: plugin.release })
    lastResult.value = `Booking Finder знайшов ${result.messages.length} листів за запитом ${result.query}.`
  })
}

watch(
  () => props.modelValue,
  value => {
    if (value) reload()
  },
  { immediate: true }
)
</script>
