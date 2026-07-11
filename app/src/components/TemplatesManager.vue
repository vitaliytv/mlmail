<script setup>
import {
  listTemplates,
  saveTemplate,
  saveBuiltinTemplate,
  deleteTemplate,
  slugify
} from '../services/newsletter-template.js'

const isDev = import.meta.env.DEV

const show = defineModel({ default: false })

const templates = ref([])
const showEditor = ref(false)
const editTemplate = ref({})

/**
 *
 */
async function load() {
  templates.value = await listTemplates()
}

watch(show, v => {
  if (v) load()
})

const saveAsBuiltin = ref(false)

/**
 *
 */
function openNew() {
  editTemplate.value = {
    id: '',
    name: '',
    type: 'newsletter',
    task_label: '',
    from_pattern: '',
    subject_pattern: '',
    prompt: ''
  }
  saveAsBuiltin.value = false
  showEditor.value = true
}

/**
 *
 * @param t
 */
function openEdit(t) {
  editTemplate.value = { task_label: '', subject_pattern: '', ...t }
  saveAsBuiltin.value = false
  showEditor.value = true
}

/**
 *
 */
async function onSave() {
  if (!editTemplate.value.id) {
    editTemplate.value.id = slugify(editTemplate.value.name || editTemplate.value.from_pattern)
  }
  if (saveAsBuiltin.value) {
    try {
      await saveBuiltinTemplate(editTemplate.value)
    } catch (error) {
      console.error('saveBuiltinTemplate failed:', error)
      alert(`Помилка збереження системного шаблону: ${error}`)
      return
    }
    // Remove user-level override so the new builtin is not shadowed.
    await deleteTemplate(editTemplate.value.id).catch(() => {})
  } else {
    await saveTemplate(editTemplate.value)
  }
  showEditor.value = false
  await load()
}

/**
 *
 * @param t
 */
async function onDelete(t) {
  await deleteTemplate(t.id)
  await load()
}

const typeLabel = t => (t.type === 'task' ? 'Завдання' : 'Newsletter')
const typeColor = t => (t.type === 'task' ? 'orange' : 'primary')
const canSave = computed(
  () =>
    editTemplate.value.name &&
    (editTemplate.value.from_pattern || editTemplate.value.subject_pattern) &&
    (editTemplate.value.type === 'task' || editTemplate.value.prompt)
)
</script>

<template>
  <q-dialog v-model="show" maximized>
    <q-card>
      <q-bar>
        <span class="text-weight-medium">Шаблони</span>
        <q-space />
        <q-btn v-close-popup flat round dense icon="sym_o_close" />
      </q-bar>

      <q-card-section class="row justify-end q-pb-none">
        <q-btn @click="openNew" color="primary" icon="sym_o_add" label="Новий шаблон" no-caps />
      </q-card-section>

      <q-card-section>
        <div v-if="!templates.length" class="text-grey-6">Шаблонів ще немає.</div>
        <q-list v-else bordered separator rounded>
          <q-item v-for="t in templates" :key="t.id">
            <q-item-section>
              <q-item-label class="row items-center q-gutter-xs">
                <span>{{ t.name }}</span>
                <q-badge :color="typeColor(t)" :label="typeLabel(t)" />
                <q-badge v-if="t.builtin" color="grey-6" label="Системний" />
                <q-badge v-else color="teal" label="Власний" />
              </q-item-label>
              <q-item-label caption>
                <span v-if="t.from_pattern">від: {{ t.from_pattern }}</span>
                <span v-if="t.from_pattern && t.subject_pattern"> · </span>
                <span v-if="t.subject_pattern">тема: {{ t.subject_pattern }}</span>
              </q-item-label>
              <q-item-label v-if="t.type === 'task' && t.task_label" caption class="text-orange">
                {{ t.task_label }}
              </q-item-label>
            </q-item-section>
            <q-item-section side>
              <div class="row q-gutter-xs">
                <q-btn @click="openEdit(t)" flat dense round icon="sym_o_edit" color="grey-7" />
                <q-btn @click="onDelete(t)" flat dense round icon="sym_o_delete" color="negative" :disable="t.builtin">
                  <q-tooltip v-if="t.builtin">Системний шаблон не можна видалити</q-tooltip>
                </q-btn>
              </div>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>
  </q-dialog>

  <!-- Editor dialog -->
  <q-dialog v-model="showEditor">
    <q-card style="min-width: 440px; max-width: 92vw">
      <q-card-section class="text-h6">
        {{ editTemplate.id ? 'Редагувати шаблон' : 'Новий шаблон' }}
      </q-card-section>
      <q-card-section class="q-gutter-md">
        <q-input v-model="editTemplate.name" label="Назва" dense outlined />
        <q-btn-toggle
          v-model="editTemplate.type"
          :options="[
            { label: 'Newsletter', value: 'newsletter' },
            { label: 'Завдання', value: 'task' }
          ]"
          dense
          no-caps
          rounded
          color="grey-7"
          text-color="white"
          toggle-color="primary" />
        <q-input
          v-model="editTemplate.from_pattern"
          label="Відправник (підрядок email, необов'язково)"
          hint="Напр.: hackernoon.com"
          dense
          outlined />
        <q-input v-model="editTemplate.subject_pattern" label="Тема листа (підрядок, необов'язково)" dense outlined />
        <q-input
          v-if="editTemplate.type === 'task'"
          v-model="editTemplate.task_label"
          label="Назва завдання"
          hint="Напр.: Здати показання лічильників"
          dense
          outlined />
        <q-input
          v-if="editTemplate.type === 'newsletter'"
          v-model="editTemplate.prompt"
          label="Інструкція для LLM"
          type="textarea"
          autogrow
          dense
          outlined />
      </q-card-section>
      <q-card-section v-if="isDev" class="q-pt-none">
        <q-toggle v-model="saveAsBuiltin" label="Зберегти як системний (dev)" color="orange" dense />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn v-close-popup flat no-caps label="Скасувати" />
        <q-btn @click="onSave" flat no-caps color="primary" label="Зберегти" :disable="!canSave" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
