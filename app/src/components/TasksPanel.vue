<script setup>
defineProps({
  tasks: { type: Array, required: true },
  isScanning: { type: Boolean, default: false },
  scannedCount: { type: Number, default: 0 },
  totalCount: { type: Number, default: 0 }
})

const emit = defineEmits(['complete-task'])

const show = ref(false)
</script>

<template>
  <!-- Floating button with badge -->
  <q-page-sticky position="bottom-right" :offset="[24, 80]">
    <q-btn @click="show = true" round color="primary" icon="sym_o_task_alt" size="md">
      <q-badge v-if="isScanning" floating color="orange" rounded>
        <q-spinner size="10px" color="white" />
      </q-badge>
      <q-badge v-else-if="tasks.length > 0" floating color="negative" :label="tasks.length" />
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
        <div v-if="!isScanning && tasks.length === 0" class="text-grey-6 q-pa-md">Відкритих завдань не знайдено.</div>

        <q-list v-else separator>
          <q-item v-for="msg in tasks" :key="msg.id">
            <q-item-section>
              <q-item-label>{{ msg.subject }}</q-item-label>
              <q-item-label caption>{{ msg.from }} · {{ msg.date }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-btn
                @click="emit('complete-task', msg.id)"
                flat
                dense
                no-caps
                color="positive"
                icon="sym_o_check_circle"
                label="Виконано" />
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>
