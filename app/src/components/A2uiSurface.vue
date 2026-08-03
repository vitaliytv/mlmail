<template>
  <!-- A2UI v1.0 surface renderer (nitra.core catalog only) -->
  <div class="a2ui-surface" data-testid="a2ui-surface">
    <div v-if="error" class="a2ui-surface__error text-negative" data-testid="a2ui-error">
      {{ error }}
    </div>
    <A2uiNode v-else-if="rootId" :node-id="rootId" :by-id="byId" :data-model="dataModel" @action="onAction" />
  </div>
</template>

<script setup>
/**
 * Renders a validated A2UI surface. Unknown catalogs/components produce an
 * error state — never HTML fallback. Prefers a pre-built `surface` object
 * (from Rust plugin-a2ui); alternatively accepts a `messages` stream and
 * validates it with the JS mirror.
 */
import { indexComponents } from '../a2ui/catalog.js'
import { validateA2uiStream } from '../a2ui/validate.js'
import A2uiNode from './A2uiNode.vue'

const props = defineProps({
  /** Pre-validated surface: { surfaceId, catalogId, components, dataModel } */
  surface: { type: Object, default: null },
  /** Raw A2UI message list (validated in-component when `surface` is absent). */
  messages: { type: Array, default: null },
  surfaceId: { type: String, default: '' }
})

const emit = defineEmits(['action', 'error'])

const parsed = computed(() => {
  if (props.surface) {
    return {
      ok: true,
      surface: props.surface,
      error: null
    }
  }
  if (!props.messages) {
    return { ok: false, surface: null, error: 'no surface or messages' }
  }
  const result = validateA2uiStream(props.messages)
  if (!result.ok) {
    return { ok: false, surface: null, error: result.error }
  }
  const id = props.surfaceId || [...result.surfaces.keys()][0]
  const s = result.surfaces.get(id)
  if (!s) {
    return { ok: false, surface: null, error: `surface not found: ${id}` }
  }
  return {
    ok: true,
    surface: {
      surfaceId: id,
      catalogId: s.catalogId,
      components: Object.fromEntries(s.components),
      dataModel: s.dataModel
    },
    error: null
  }
})

const error = computed(() => parsed.value.error)

watch(
  error,
  e => {
    if (e) emit('error', e)
  },
  { immediate: true }
)

const byId = computed(() => indexComponents(parsed.value.surface?.components || {}))
const dataModel = computed(() => parsed.value.surface?.dataModel || {})
const rootId = computed(() => (byId.value.has('root') ? 'root' : null))

function onAction(payload) {
  emit('action', payload)
}
</script>

<style scoped>
.a2ui-surface {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.a2ui-surface__error {
  font-size: 13px;
  padding: 8px 0;
}
</style>
