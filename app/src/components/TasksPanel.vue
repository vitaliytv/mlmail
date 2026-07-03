<script setup>
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  tasks: { type: Array, required: true },
  isScanning: { type: Boolean, default: false },
  scannedCount: { type: Number, default: 0 },
  totalCount: { type: Number, default: 0 },
})

const emit = defineEmits(['remove-message'])

const show = ref(false)

async function markDone(templateId, message) {
  try {
    await invoke('gmail_trash', { id: message.id })
    emit('remove-message', templateId, message.id)
  }
  catch (error) {
    console.error('trash failed', error)
  }
}
</script>

<template>
  <!-- Floating button with badge -->
  <q-page-sticky position="bottom-right" :offset="[24, 80]">
    <q-btn
      round
      color="primary"
      icon="sym_o_task_alt"
      size="md"
      @click="show = true">
      <q-badge
        v-if="isScanning"
        floating
        color="orange"
        rounded>
        <q-spinner size="10px" color="white" />
      </q-badge>
      <q-badge
        v-else-if="tasks.length > 0"
        floating
        color="negative"
        :label="tasks.reduce((s, g) => s + g.messages.length, 0)" />
    </q-btn>
  </q-page-sticky>

  <!-- Tasks dialog -->
  <q-dialog v-model="show">
    <q-card style="min-width: 520px; max-width: 92vw">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6">Відкриті завдання</div>
        <q-space />
        <q-btn v-close-popup flat round dense icon="sym_o_close" />
      </q-card-section>

      <!-- Scan progress -->
      <q-linear-progress
        v-if="isScanning"
        :value="totalCount ? scannedCount / totalCount : 0"
        color="primary"
        class="q-mt-sm" />
      <div v-if="isScanning" class="text-caption text-grey-6 q-px-md q-pb-xs">
        Сканування {{ scannedCount }} / {{ totalCount }}…
      </div>

      <q-separator />

      <q-card-section style="max-height: 65vh; overflow-y: auto" class="q-pa-none">
        <!-- Empty -->
        <div v-if="!isScanning && tasks.length === 0" class="text-grey-6 q-pa-md">
          Відкритих завдань не знайдено.
        </div>

        <!-- Task groups -->
        <div v-for="group in tasks" :key="group.template.id">
          <q-item-label header class="text-primary">
            {{ group.template.task_label || group.template.name }}
          </q-item-label>
          <q-list separator>
            <q-item v-for="msg in group.messages" :key="msg.id">
              <q-item-section>
                <q-item-label>{{ msg.subject }}</q-item-label>
                <q-item-label caption>{{ msg.from }} · {{ msg.date }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-btn
                  flat dense no-caps
                  color="positive"
                  icon="sym_o_check_circle"
                  label="Виконано"
                  @click="markDone(group.template.id, msg)" />
              </q-item-section>
            </q-item>
          </q-list>
        </div>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>
