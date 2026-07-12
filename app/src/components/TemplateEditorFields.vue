<script setup>
const model = defineModel({ type: Object, required: true })
defineProps({
  subjectHint: { type: String, default: '' },
  promptHint: { type: String, default: '' }
})
</script>

<template>
  <q-input v-model="model.name" label="Назва" dense outlined />
  <q-btn-toggle
    v-model="model.type"
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
    v-model="model.from_pattern"
    label="Відправник (підрядок email, необов'язково)"
    hint="Напр.: hackernoon.com"
    dense
    outlined />
  <q-input
    v-model="model.subject_pattern"
    label="Тема листа (підрядок, необов'язково)"
    :hint="subjectHint"
    dense
    outlined />
  <q-input
    v-if="model.type === 'task'"
    v-model="model.task_label"
    label="Назва завдання"
    hint="Напр.: Здати показання лічильників"
    dense
    outlined />
  <q-input
    v-if="model.type === 'newsletter'"
    v-model="model.prompt"
    label="Інструкція для LLM"
    type="textarea"
    autogrow
    dense
    outlined
    :hint="promptHint" />
</template>
