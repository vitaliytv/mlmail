<template>
  <component :is="tag" v-bind="attrs" @click="onClick">
    <template v-if="isLayout">
      <A2uiNode
        v-for="cid in childIds"
        :key="cid"
        :node-id="cid"
        :by-id="byId"
        :data-model="dataModel"
        @action="forwardAction" />
    </template>
    <template v-else-if="node.component === 'Button'">
      <A2uiNode v-if="node.child" :node-id="node.child" :by-id="byId" :data-model="dataModel" @action="forwardAction" />
    </template>
    <template v-else-if="node.component === 'Text'">
      {{ textContent }}
    </template>
  </component>
</template>

<script setup>
/**
 * Recursive nitra.core node. Unrecognized component types render nothing
 * (parent surface already validated; this is a last-line guard).
 */
import { resolveDynamicString } from '../a2ui/catalog.js'
// Self-import for recursive children (script setup).
import A2uiNode from './A2uiNode.vue'

const props = defineProps({
  nodeId: { type: String, required: true },
  byId: { type: Map, required: true },
  dataModel: { type: Object, default: () => ({}) }
})

const emit = defineEmits(['action'])

const node = computed(() => props.byId.get(props.nodeId) || { component: 'Unknown' })

const isLayout = computed(() => node.value.component === 'Column' || node.value.component === 'Row')

const childIds = computed(() => {
  const ch = node.value.children
  return Array.isArray(ch) ? ch : []
})

const textContent = computed(() => resolveDynamicString(node.value.text, props.dataModel))

const tag = computed(() => {
  switch (node.value.component) {
    case 'Column':
      return 'div'
    case 'Row':
      return 'div'
    case 'Text':
      return 'p'
    case 'Button':
      return 'button'
    case 'Divider':
      return 'hr'
    default:
      return 'span'
  }
})

const attrs = computed(() => {
  const n = node.value
  const base = { 'data-a2ui-id': props.nodeId, 'data-a2ui-component': n.component }
  if (n.component === 'Column') {
    return { ...base, class: 'a2ui-column', style: columnStyle(n) }
  }
  if (n.component === 'Row') {
    return { ...base, class: 'a2ui-row', style: rowStyle(n) }
  }
  if (n.component === 'Text') {
    return {
      ...base,
      class: n.variant === 'caption' ? 'a2ui-text a2ui-text--caption' : 'a2ui-text'
    }
  }
  if (n.component === 'Button') {
    return {
      ...base,
      type: 'button',
      class: buttonClass(n.variant)
    }
  }
  if (n.component === 'Divider') {
    return { ...base, class: 'a2ui-divider' }
  }
  return { ...base, class: 'a2ui-unknown', style: { display: 'none' } }
})

function columnStyle(n) {
  return {
    display: 'flex',
    flexDirection: 'column',
    alignItems: cssAlign(n.align),
    justifyContent: cssJustify(n.justify),
    gap: '8px'
  }
}

function rowStyle(n) {
  return {
    display: 'flex',
    flexDirection: 'row',
    alignItems: cssAlign(n.align),
    justifyContent: cssJustify(n.justify),
    gap: '8px',
    flexWrap: 'wrap'
  }
}

function cssAlign(v) {
  return (
    {
      start: 'flex-start',
      center: 'center',
      end: 'flex-end',
      stretch: 'stretch'
    }[v] || 'stretch'
  )
}

function cssJustify(v) {
  return (
    {
      start: 'flex-start',
      center: 'center',
      end: 'flex-end',
      spaceBetween: 'space-between',
      spaceAround: 'space-around',
      spaceEvenly: 'space-evenly',
      stretch: 'flex-start'
    }[v] || 'flex-start'
  )
}

function buttonClass(variant) {
  if (variant === 'primary') return 'a2ui-btn a2ui-btn--primary'
  if (variant === 'borderless') return 'a2ui-btn a2ui-btn--borderless'
  return 'a2ui-btn'
}

function onClick() {
  if (node.value.component !== 'Button') return
  const action = node.value.action
  emit('action', {
    surfaceComponentId: props.nodeId,
    action
  })
}

function forwardAction(payload) {
  emit('action', payload)
}
</script>

<style scoped>
.a2ui-text {
  margin: 0;
  font-size: 14px;
  line-height: 1.4;
}

.a2ui-text--caption {
  font-size: 12px;
  opacity: 0.75;
}

.a2ui-btn {
  appearance: none;
  border: 1px solid color-mix(in srgb, currentColor 28%, transparent);
  background: color-mix(in srgb, currentColor 8%, transparent);
  border-radius: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font: inherit;
}

.a2ui-btn--primary {
  background: var(--q-primary, #1976d2);
  border-color: transparent;
  color: #fff;
}

.a2ui-btn--borderless {
  border-color: transparent;
  background: transparent;
  text-decoration: underline;
}

.a2ui-divider {
  border: 0;
  border-top: 1px solid color-mix(in srgb, currentColor 18%, transparent);
  margin: 4px 0;
  width: 100%;
}
</style>
