<template>
  <BaseDialog
    @update:model-value="val => emit('update:modelValue', val)"
    @show="onShow"
    :model-value="modelValue"
    title="Request journal"
    icon="sym_o_history"
    :width="720"
    body-class="">
    <template #header>
      <q-btn @click="refresh" icon="sym_o_refresh" flat round dense size="sm" :loading="loading" title="Refresh" />
    </template>

    <div v-if="!records.length && !loading" class="audit-empty">No requests yet</div>

    <div v-for="rec in records" :key="rec.id" class="audit-row">
      <div @click="toggle(rec.id)" class="audit-head">
        <StatePill :status="rec.status" />
        <span class="audit-actor">{{ rec.actor?.kind }}{{ rec.actor?.id ? `:${rec.actor.id}` : '' }}</span>
        <span class="audit-intent">{{ rec.intent }}</span>
        <q-space />
        <span class="audit-time">{{ fmtTime(rec.createdAt) }}</span>
      </div>

      <div v-if="expandedId === rec.id" class="audit-body">
        <RequestView @respond="msg => onRespond(rec, msg)" :result="rec" :busy="busyId === rec.id" />
        <div v-if="rec.status === 'needs_approval' && rec.pendingApproval" class="audit-approval">
          <div class="audit-pending">
            Approve action: <code>{{ rec.pendingApproval.tool }}({{ JSON.stringify(rec.pendingApproval.input) }})</code>
          </div>
          <div class="row q-gutter-sm">
            <q-btn
              @click="onApprove(rec, true)"
              label="Підтвердити"
              color="negative"
              unelevated
              no-caps
              :loading="busyId === rec.id" />
            <q-btn @click="onApprove(rec, false)" label="Відхилити" flat no-caps :disable="busyId === rec.id" />
          </div>
        </div>

        <div class="audit-analysis">
          <div class="row q-gutter-sm">
            <q-btn
              @click="runAnalysis(rec, 'pi')"
              label="Аналіз: pi (cloud max)"
              icon="sym_o_travel_explore"
              flat
              no-caps
              dense
              size="sm"
              :loading="analyzingId === rec.id && analyzingVia === 'pi'"
              :disable="analyzingId === rec.id" />
            <q-btn
              @click="runAnalysis(rec, 'omlx')"
              label="Аналіз: omlx"
              icon="sym_o_dns"
              flat
              no-caps
              dense
              size="sm"
              :loading="analyzingId === rec.id && analyzingVia === 'omlx'"
              :disable="analyzingId === rec.id" />
          </div>
          <div v-if="analysisError[rec.id]" class="audit-analysis-error">{{ analysisError[rec.id] }}</div>
          <div v-if="analysisResult[rec.id]" class="audit-analysis-result">{{ analysisResult[rec.id] }}</div>
        </div>
      </div>
    </div>
  </BaseDialog>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { useQuasar } from 'quasar'
import { BaseDialog, RequestView, StatePill } from '@7n/tauri-components/components'
import { useCallAnalysis } from '../composables/use-call-analysis.js'

// Local fork of @7n/tauri-components' AuditDialog: that component has no
// slots to hang extra per-row actions off of, and it ships from a separate
// npm-released repo, so the "analyze this call" buttons live here instead —
// same journal data source, same look, plus an analysis panel per row.
const props = defineProps({
  modelValue: { type: Boolean, default: false },
  agent: { type: Object, required: true }
})
const emit = defineEmits(['update:modelValue', 'changed'])

const $q = useQuasar()
const { journal, respond, approve, loadOmlxEnv } = props.agent
const { analyzeWithPi, analyzeWithOmlx } = useCallAnalysis(props.agent)

const records = ref([])
const loading = ref(false)
const expandedId = ref(null)
const busyId = ref(null)
const analyzingId = ref(null)
const analyzingVia = ref(null)
const analysisResult = reactive({})
const analysisError = reactive({})

/**
 * @param {number} millis epoch millis
 * @returns {string} locale time
 */
function fmtTime(millis) {
  return millis ? new Date(millis).toLocaleString() : ''
}

/**
 * Reload the journal list.
 */
async function refresh() {
  loading.value = true
  try {
    records.value = await journal.list()
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error?.message ?? error) })
  } finally {
    loading.value = false
  }
}

/**
 * Load the omlx config (base URL/model/key) before the list, so the omlx
 * analysis button works even if the agent dialog was never opened first.
 */
async function onShow() {
  await loadOmlxEnv()
  await refresh()
}

/**
 * @param {string} id record id to expand/collapse
 */
function toggle(id) {
  expandedId.value = expandedId.value === id ? null : id
}

/**
 * Answer a pending clarification on a journaled request, then refresh.
 * @param {object} rec the record
 * @param {string} message the human's answer
 */
async function onRespond(rec, message) {
  busyId.value = rec.id
  try {
    await respond(rec.id, message)
    await refresh()
    emit('changed')
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error?.message ?? error) })
  } finally {
    busyId.value = null
  }
}

/**
 * Approve or reject a pending destructive action, then refresh.
 * @param {object} rec the record
 * @param {boolean} ok true to approve (execute), false to reject
 */
async function onApprove(rec, ok) {
  busyId.value = rec.id
  try {
    await approve(rec.id, ok)
    await refresh()
    emit('changed')
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error?.message ?? error) })
  } finally {
    busyId.value = null
  }
}

/**
 * Ask pi (cloud max model) or the local omlx model to analyze this call's
 * request/response and suggest project-code adaptations.
 * @param {object} rec the journal record to analyze
 * @param {'pi'|'omlx'} via which backend to use
 */
async function runAnalysis(rec, via) {
  analyzingId.value = rec.id
  analyzingVia.value = via
  analysisError[rec.id] = ''
  analysisResult[rec.id] = ''
  try {
    analysisResult[rec.id] = via === 'pi' ? await analyzeWithPi(rec) : await analyzeWithOmlx(rec)
  } catch (error) {
    analysisError[rec.id] = String(error?.message ?? error)
  } finally {
    analyzingId.value = null
    analyzingVia.value = null
  }
}
</script>

<style scoped>
.audit-empty {
  text-align: center;
  padding: 40px 0;
  font-size: 13px;
  opacity: 0.4;
}

.audit-approval {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
  padding: 10px;
  border-radius: 8px;
  background: color-mix(in srgb, #ff9f0a 12%, transparent);
}

.audit-pending {
  font-size: 13px;
  overflow-wrap: anywhere;
}

.audit-row {
  border-bottom: 1px solid rgb(255 255 255 / 7%);
}

.body--light .audit-row {
  border-bottom-color: rgb(0 0 0 / 7%);
}

.audit-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 4px;
  cursor: pointer;
}

.audit-head:hover {
  background: rgb(255 255 255 / 5%);
}

.body--light .audit-head:hover {
  background: rgb(0 0 0 / 4%);
}

.audit-actor {
  font-family: 'SF Mono', ui-monospace, 'JetBrains Mono', monospace;
  font-size: 11px;
  opacity: 0.6;
  flex: 0 0 auto;
}

.audit-intent {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1 1 auto;
  min-width: 0;
}

.audit-time {
  font-size: 11px;
  opacity: 0.4;
  flex: 0 0 auto;
}

.audit-body {
  padding: 6px 4px 12px;
}

.audit-analysis {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}

.audit-analysis-error {
  color: #ff6b60;
  font-size: 13px;
}

.audit-analysis-result {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-size: 13px;
  line-height: 1.5;
  padding: 10px;
  border-radius: 8px;
  background: rgb(255 255 255 / 5%);
}

.body--light .audit-analysis-result {
  background: rgb(0 0 0 / 4%);
}
</style>
