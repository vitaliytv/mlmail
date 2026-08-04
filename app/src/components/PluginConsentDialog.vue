<template>
  <q-dialog :model-value="modelValue" @update:model-value="$emit('update:modelValue', $event)">
    <q-card style="min-width: 480px; max-width: 90vw">
      <q-card-section class="text-h6 row items-center">
        Дозволи плагіна
        <q-space />
        <q-btn v-close-popup flat round dense icon="sym_o_close" />
      </q-card-section>
      <q-separator />
      <q-card-section v-if="preview">
        <div class="text-subtitle1">{{ preview.manifest.name }} · {{ preview.manifest.version }}</div>
        <div class="text-caption text-grey-7">
          {{ preview.manifest.publisher }} · {{ preview.manifest.publisherKeyId || preview.manifest.publisher_key_id }}
        </div>
        <div v-if="preview.fingerprint" class="text-caption q-mt-sm">
          Fingerprint: <code>{{ preview.fingerprint }}</code>
        </div>
        <div v-if="preview.diff.keyChanged" class="text-negative q-mt-sm">
          Ключ видавця змінився — потрібне повторне підтвердження.
        </div>

        <div class="q-mt-md text-weight-medium">Нові / розширені</div>
        <q-list v-if="preview.diff.added.length" dense bordered class="q-mt-xs">
          <q-item v-for="(c, i) in preview.diff.added" :key="'a' + i">
            <q-item-section>{{ capLabel(c) }}</q-item-section>
          </q-item>
        </q-list>
        <div v-else class="text-caption text-grey-6">Немає нових capability.</div>

        <div class="q-mt-md text-weight-medium">Без змін</div>
        <div class="text-caption text-grey-6">{{ preview.diff.unchanged.length }} capability</div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn v-close-popup flat label="Скасувати" />
        <q-btn @click="accept" color="primary" label="Дозволити й встановити" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup>
/**
 * Consent dialog: shows capability diff + TOFU fingerprint before install.
 */
defineProps({
  modelValue: { type: Boolean, default: false },
  preview: { type: Object, default: null }
})

const emit = defineEmits(['update:modelValue', 'accept'])

function capLabel(c) {
  const kinds = (c.resourceKinds || c.resource_kinds || []).join(', ')
  return `${c.name}${kinds ? ` (${kinds})` : ''}`
}

function accept() {
  emit('accept')
}
</script>
