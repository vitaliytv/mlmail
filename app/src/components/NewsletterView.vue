<script setup>
import { useQuasar } from 'quasar'
import { useNewsletterRender } from '../composables/use-newsletter-render.js'
import { useSummary } from '../composables/use-summary.js'
import { useAsk } from '../composables/use-ask.js'
import { useTaskScan } from '../composables/use-task-scan.js'
import { usePattern } from '../composables/use-pattern.js'
import { useAuthStore } from '../services/auth-store.js'
import { buildPatternQuery, parseFromEmail } from '../services/pattern.js'
import { errorMessage } from '../i18n/auth-errors.js'
import TemplateEditorFields from './TemplateEditorFields.vue'
import {
  listTemplates,
  saveTemplate,
  deleteTemplate,
  findTemplateForMessage,
  slugify
} from '../services/newsletter-template.js'

const FROM_DOMAIN_RE = /@([\w.-]+)>?/

const props = defineProps({
  message: { type: Object, required: true }
})

const $q = useQuasar()
const auth = useAuthStore()
const renderer = useNewsletterRender()
const summary = useSummary()
const asker = useAsk()
const taskScan = useTaskScan()
const pattern = usePattern()

const askQuestion = ref('')
const askAnswer = ref('')
const askFailed = ref(false)

/**
 *
 */
async function submitAsk() {
  if (!askQuestion.value.trim()) return
  askAnswer.value = ''
  askFailed.value = false
  const result = await asker.ask(props.message, askQuestion.value)
  if (result === null) {
    askFailed.value = true
  } else {
    askAnswer.value = result
    askQuestion.value = ''
  }
}

// Template state
const templates = ref([])
const activeTemplate = ref(null)

// Template render state
const articles = ref(/** @type {import('../services/newsletter-template.js').NewsletterArticle[]} */ ([]))
const isRendering = ref(false)
const renderFailed = ref(false)

// Summary/translate fallback state
const rightMode = ref(/** @type {'summary'|'translate'} */ ('summary'))
const summaryText = ref('')
const isSummarizing = ref(false)
const summaryFailed = ref(false)
const translateHtml = ref('')
const isTranslating = ref(false)
const translateFailed = ref(false)

// Template editor dialog
const showEditor = ref(false)
const editTemplate = ref({ id: '', name: '', from_pattern: '', subject_pattern: '', prompt: '' })

// Rule-from-this-email panel state
const patternFrom = ref('')
const patternSubject = ref('')
const initialSubject = ref('')
const isSuggesting = ref(false)

const patternQuery = computed(() => buildPatternQuery({ from: patternFrom.value, subject: patternSubject.value }))
// Deletion targets the sender only — a subject phrase (even an LLM-suggested
// one) is too easy to get wrong and trash unrelated mail from the same sender.
const deleteQuery = computed(() => buildPatternQuery({ from: patternFrom.value }))

let suggestionToken = 0

// Per-message result cache so flipping back to an already-processed email
// shows the previous summary/translation instead of re-running the LLM.
// Only successful results are cached — a failure should still be retryable.
const summaryCache = new Map()
const translateCache = new Map()

/**
 *
 */
async function loadTemplates() {
  templates.value = await listTemplates()
  // Only newsletter-type templates activate the article extractor.
  const newsletters = templates.value.filter(t => !t.type || t.type === 'newsletter')
  activeTemplate.value = findTemplateForMessage(props.message, newsletters)
}

/**
 * @param {import('../services/newsletter-template.js').NewsletterTemplate | null} template
 *   the matched newsletter template, or null to fall back to plain summarization
 */
async function refreshRender(template) {
  articles.value = []
  renderFailed.value = false
  summaryText.value = ''
  summaryFailed.value = false

  if (template) {
    isRendering.value = true
    const result = await renderer.render(props.message, template.prompt)
    isRendering.value = false
    if (result === null) renderFailed.value = true
    else articles.value = result
  } else {
    if (!(props.message?.body ?? '').trim()) return
    const id = props.message.id
    if (summaryCache.has(id)) {
      summaryText.value = summaryCache.get(id)
      return
    }
    isSummarizing.value = true
    const text = await summary.summarize(props.message)
    isSummarizing.value = false
    summaryFailed.value = text === null
    if (text !== null) {
      summaryText.value = text
      summaryCache.set(id, text)
    }
  }
}

/**
 *
 */
async function refreshTranslate() {
  const id = props.message.id
  if (translateCache.has(id)) {
    translateHtml.value = translateCache.get(id)
    translateFailed.value = false
    return
  }
  translateHtml.value = ''
  translateFailed.value = false
  isTranslating.value = true
  const result = await summary.translateHtml(props.message)
  isTranslating.value = false
  translateFailed.value = result === null
  if (result !== null) {
    translateHtml.value = result.html
    translateCache.set(id, result.html)
  }
}

/**
 * @param {string} mode the newly selected right-panel mode
 */
async function onModeChange(mode) {
  if (mode === 'translate' && !isTranslating.value) {
    await refreshTranslate()
  }
}

/**
 * Seed the "rule from this email" fields from the current message, then
 * refine the subject with a local-LLM suggestion (best-effort).
 * @param {object} msg the current message
 */
async function seedPattern(msg) {
  auth.clearPatternFeedback()
  patternFrom.value = parseFromEmail(msg.from)
  patternSubject.value = (msg.subject ?? '').trim()
  initialSubject.value = patternSubject.value
  isSuggesting.value = true
  const token = ++suggestionToken
  const suggestion = await pattern.suggestSubjectPattern(msg.subject)
  // Ignore the result if it was cancelled, the message changed, or the user
  // edited the field while we were waiting.
  if (!Object.is(token, suggestionToken)) return
  if (Object.is(patternSubject.value, initialSubject.value)) {
    patternSubject.value = suggestion
  }
  isSuggesting.value = false
}

/**
 * Cancel a pending subject suggestion: drop its result when it arrives and
 * clear the field immediately instead of waiting for the LLM.
 */
function cancelSuggestion() {
  suggestionToken++
  isSuggesting.value = false
  patternSubject.value = ''
}

/** Trash all mail matching the delete query, then toast the result. */
async function trashByQueryAndNotify() {
  const q = deleteQuery.value
  await auth.trashByQuery(q)
  if (!auth.trashQueryErrorKind.value) {
    const count = auth.lastTrashedCount.value ?? 0
    $q.notify({
      message: `Переміщено в кошик: ${count}`,
      caption: q,
      color: 'positive',
      icon: 'sym_o_delete_sweep',
      timeout: 5000,
      position: 'top'
    })
  }
}

watch(
  () => props.message,
  async () => {
    askAnswer.value = ''
    askFailed.value = false
    askQuestion.value = ''
    translateHtml.value = ''
    translateFailed.value = false
    seedPattern(props.message)
    await loadTemplates()
    await refreshRender(activeTemplate.value)
    if (!activeTemplate.value && rightMode.value === 'translate') {
      await refreshTranslate()
    }
  },
  { immediate: true }
)

watch(activeTemplate, refreshRender)

/**
 *
 */
function openNewTemplate() {
  const from = props.message?.from ?? ''
  const domain = from.match(FROM_DOMAIN_RE)?.[1] ?? ''
  editTemplate.value = {
    id: slugify(domain || from),
    name: domain || from,
    type: 'newsletter',
    task_label: '',
    from_pattern: domain || '',
    subject_pattern: '',
    prompt:
      'Extract all article titles and their "Read more" links. Also extract items from "Additional Stories" section. For each article return its title, url, and a 1-2 sentence description.'
  }
  showEditor.value = true
}

/**
 *
 */
function openEditTemplate() {
  editTemplate.value = { subject_pattern: '', task_label: '', type: 'newsletter', ...activeTemplate.value }
  showEditor.value = true
}

/** Apply the "Задача" Gmail label to the current message. */
async function flagAsTask() {
  await taskScan.flagMessage(props.message.id)
}

/**
 *
 */
async function onSaveTemplate() {
  await saveTemplate(editTemplate.value)
  showEditor.value = false
  await loadTemplates()
  await refreshRender(activeTemplate.value)
  if (editTemplate.value.type === 'task') {
    await taskScan.scan(templates.value)
  }
}

/**
 *
 */
async function onDeleteTemplate() {
  await deleteTemplate(activeTemplate.value.id)
  await loadTemplates()
  await refreshRender(null)
}

defineExpose({ flagAsTask })
</script>

<template>
  <div class="column fit">
    <!-- Header row -->
    <div class="row items-center q-px-md q-pt-sm q-pb-xs">
      <div class="text-overline text-grey-7">
        {{
          activeTemplate
            ? activeTemplate.name
            : rightMode === 'translate'
              ? 'Переклад українською'
              : 'Резюме українською'
        }}
      </div>
      <q-space />
      <q-spinner v-if="isRendering || isSummarizing || isTranslating" color="primary" size="18px" class="q-mr-xs" />
      <q-btn-toggle
        v-if="!activeTemplate"
        v-model="rightMode"
        @update:model-value="onModeChange"
        :options="[
          { value: 'summary', icon: 'sym_o_summarize' },
          { value: 'translate', icon: 'sym_o_translate' }
        ]"
        dense
        flat
        no-caps
        rounded
        color="grey-6"
        toggle-color="primary"
        size="sm"
        class="q-mr-xs" />
      <q-btn
        v-if="activeTemplate"
        @click="openEditTemplate"
        flat
        dense
        round
        icon="sym_o_edit"
        size="sm"
        color="grey-6"
        title="Редагувати шаблон" />
      <q-btn
        v-if="activeTemplate"
        @click="onDeleteTemplate"
        flat
        dense
        round
        icon="sym_o_delete"
        size="sm"
        color="grey-6"
        title="Видалити шаблон" />
      <q-btn
        v-if="!activeTemplate"
        @click="openNewTemplate"
        flat
        dense
        round
        icon="sym_o_add"
        size="sm"
        color="grey-6"
        title="Створити шаблон для цього відправника" />
    </div>
    <q-separator inset />

    <!-- Content -->
    <div class="col overflow-auto q-px-md q-pb-md">
      <!-- === Template mode === -->
      <template v-if="activeTemplate">
        <template v-if="isRendering">
          <div v-for="i in 3" :key="i" class="q-pt-md">
            <q-skeleton type="text" width="60%" />
            <q-skeleton type="text" />
            <q-skeleton type="text" width="80%" />
          </div>
        </template>
        <q-banner v-else-if="renderFailed" class="bg-orange-1 text-orange-9 q-mt-md" rounded dense>
          Не вдалося отримати статті. Перевірте, чи запущено локальну модель (omlx).
        </q-banner>
        <div v-else-if="articles.length === 0" class="text-grey-6 q-pt-md text-body2">
          Шаблон не знайшов статей у цьому листі.
        </div>
        <div v-else class="q-gutter-y-sm q-pt-md">
          <div v-for="(article, i) in articles" :key="i" class="article-item">
            <a
              v-if="article.url"
              :href="article.url"
              target="_blank"
              rel="noopener noreferrer"
              class="article-title text-primary">
              {{ article.title }}
            </a>
            <div v-else class="article-title text-dark">{{ article.title }}</div>
            <div v-if="article.description" class="article-desc text-grey-8">
              {{ article.description }}
            </div>
          </div>
        </div>
      </template>

      <!-- === Summary / Translate fallback mode === -->
      <template v-else>
        <!-- Summary -->
        <template v-if="rightMode === 'summary'">
          <template v-if="isSummarizing">
            <q-skeleton type="text" class="q-mt-md" />
            <q-skeleton type="text" />
            <q-skeleton type="text" width="70%" />
          </template>
          <q-banner v-else-if="summaryFailed" class="bg-orange-1 text-orange-9 q-mt-md" rounded dense>
            Не вдалося отримати резюме. Перевірте, чи запущено локальну модель (omlx).
          </q-banner>
          <div v-else-if="summaryText" class="summary-body q-pt-md">{{ summaryText }}</div>
          <div v-else class="text-grey-6 q-pt-md">Порожній лист — нема що резюмувати.</div>
        </template>
        <!-- Translate -->
        <template v-else>
          <template v-if="isTranslating">
            <q-linear-progress
              v-if="summary.translateProgress.value.total"
              :value="summary.translateProgress.value.done / summary.translateProgress.value.total"
              size="4px"
              color="primary"
              class="q-mt-md" />
            <div v-if="summary.translateProgress.value.total" class="text-caption text-grey-6 q-mt-xs">
              Переклад: {{ summary.translateProgress.value.done }}/{{ summary.translateProgress.value.total }}
            </div>
            <q-skeleton type="text" class="q-mt-md" />
            <q-skeleton type="text" />
            <q-skeleton type="text" width="70%" />
          </template>
          <q-banner v-else-if="translateFailed" class="bg-orange-1 text-orange-9 q-mt-md" rounded dense>
            Не вдалося перекласти. Перевірте, чи запущено локальну модель (omlx).
          </q-banner>
          <iframe
            v-else-if="translateHtml"
            :srcdoc="translateHtml"
            sandbox="allow-scripts"
            class="translate-iframe"
            referrerpolicy="no-referrer" />
          <div v-else class="text-grey-6 q-pt-md">Порожній лист — нема що перекладати.</div>
        </template>
      </template>
    </div>

    <!-- Ask panel -->
    <div class="ask-panel q-px-md q-pb-sm q-pt-xs">
      <div v-if="askAnswer" class="ask-answer q-mb-sm text-body2">{{ askAnswer }}</div>
      <q-banner v-if="askFailed" class="bg-orange-1 text-orange-9 q-mb-sm" rounded dense>
        Не вдалося отримати відповідь. Перевірте локальну модель (omlx).
      </q-banner>
      <q-input
        v-model="askQuestion"
        @keydown.enter.prevent="submitAsk"
        placeholder="Запитати про цей лист…"
        dense
        outlined
        :loading="asker.isAsking.value">
        <template #append>
          <q-btn
            @click="submitAsk"
            flat
            dense
            round
            icon="sym_o_send"
            color="primary"
            :disable="!askQuestion.trim() || asker.isAsking.value" />
        </template>
      </q-input>
    </div>

    <!-- Rule-from-this-email panel -->
    <div class="rule-panel q-px-md q-pb-sm q-pt-sm">
      <div class="text-overline text-grey-7 q-mb-xs">Правило для схожих листів</div>
      <div class="q-gutter-sm">
        <q-input v-model="patternFrom" label="Від (email)" dense outlined clearable clear-icon="sym_o_close" />
        <q-input
          v-model="patternSubject"
          label="Тема містить"
          dense
          outlined
          :clearable="!isSuggesting"
          clear-icon="sym_o_close">
          <template v-if="isSuggesting" #append>
            <q-icon @click="cancelSuggestion" name="sym_o_close" class="cursor-pointer" />
          </template>
        </q-input>
        <div class="text-caption text-grey-7">
          Фільтр: <code>{{ patternQuery || '—' }}</code
          ><br />
          Видалення (лише за відправником): <code>{{ deleteQuery || '—' }}</code>
        </div>
      </div>

      <q-banner
        v-if="auth.trashQueryErrorKind.value || auth.filterErrorKind.value"
        class="bg-red-1 text-red-9 q-mt-sm"
        rounded
        dense>
        {{ errorMessage(auth.trashQueryErrorKind.value || auth.filterErrorKind.value) }}
      </q-banner>
      <q-banner v-else-if="auth.filterCreated.value" class="bg-green-1 text-green-9 q-mt-sm" rounded dense>
        Фільтр створено — нові такі листи йтимуть у кошик.
      </q-banner>

      <div class="row justify-end q-gutter-sm q-mt-sm">
        <q-btn
          @click="auth.createFilter({ from: patternFrom, subject: patternSubject })"
          flat
          no-caps
          color="primary"
          icon="sym_o_filter_alt"
          label="Створити фільтр"
          :disable="!patternQuery"
          :loading="auth.isCreatingFilter.value" />
        <q-btn
          @click="trashByQueryAndNotify"
          flat
          no-caps
          color="negative"
          icon="sym_o_delete_sweep"
          label="Видалити всі такі"
          :disable="!deleteQuery"
          :loading="auth.isTrashingQuery.value" />
      </div>
    </div>
  </div>

  <!-- Template editor dialog -->
  <q-dialog v-model="showEditor">
    <q-card style="min-width: 420px; max-width: 90vw">
      <q-card-section class="text-h6">
        {{ editTemplate.id === activeTemplate?.id ? 'Редагувати шаблон' : 'Новий шаблон' }}
      </q-card-section>
      <q-card-section class="q-gutter-md">
        <TemplateEditorFields
          v-model="editTemplate"
          subject-hint="Напр.: скаїтатаju rādījumus"
          prompt-hint="Що витягти з листа (назви статей, посилання, розділи тощо)" />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn v-close-popup flat no-caps label="Скасувати" />
        <q-btn
          @click="onSaveTemplate"
          flat
          no-caps
          color="primary"
          label="Зберегти"
          :disable="
            (!editTemplate.from_pattern && !editTemplate.subject_pattern) ||
            (editTemplate.type === 'newsletter' && !editTemplate.prompt)
          " />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<style scoped>
.article-item {
  border-left: 3px solid var(--q-primary);
  padding-left: 10px;
}

.article-title {
  font-weight: 600;
  font-size: 0.9rem;
  text-decoration: none;
  display: block;
}

.article-title:hover {
  text-decoration: underline;
}

.article-desc {
  font-size: 0.82rem;
  margin-top: 2px;
  line-height: 1.4;
}

.summary-body {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: inherit;
}

.translate-iframe {
  width: 100%;
  min-height: 600px;
  border: none;
  display: block;
}

.ask-panel {
  border-top: 1px solid rgb(128 128 128 / 20%);
}

.rule-panel {
  border-top: 1px solid rgb(128 128 128 / 20%);
}

.ask-answer {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  padding: 8px 10px;
  border-left: 3px solid var(--q-primary);
  background: rgb(var(--q-primary-rgb, 25, 118, 210), 0.06);
  border-radius: 4px;
}
</style>
