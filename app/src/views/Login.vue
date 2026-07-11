<script setup>
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { useQuasar } from 'quasar'
import { AgentDialog } from '@7n/tauri-components/components'
import { errorMessage } from '../i18n/auth-errors.js'
import { useAuthStore } from '../services/auth-store.js'
import { useAgent } from '../composables/use-agent.js'
import { usePattern } from '../composables/use-pattern.js'
import { buildPatternQuery, parseFromEmail } from '../services/pattern.js'
import AuditAnalysisDialog from '../components/AuditAnalysisDialog.vue'
import NewsletterView from '../components/NewsletterView.vue'
import TemplatesManager from '../components/TemplatesManager.vue'
import GmailFiltersDialog from '../components/GmailFiltersDialog.vue'

const auth = useAuthStore()
const agent = useAgent()
const newsletterViewRef = ref(null)

const appVersion = ref('')
onMounted(async () => {
  appVersion.value = await getVersion()
})

watchEffect(() => {
  const email = auth.email.value
  const count = auth.inboxCount.value
  const version = appVersion.value
  const appName = version ? `mlmail v${version}` : 'mlmail'
  let title
  if (email && count !== null) {
    title = `${appName} - ${email} - ${count}`
  } else if (email) {
    title = `${appName} - ${email}`
  } else {
    title = appName
  }
  document.title = title
  invoke('app_set_title', { title })
})
const pattern = usePattern()
const agentOpen = ref(false)
const auditOpen = ref(false)
const $q = useQuasar()
onMounted(() => {
  auth.initialize()
  window.addEventListener('message', (e) => {
    if (e.data?.type === 'open-url' && e.data.url) {
      invoke('app_open_url', { url: e.data.url })
    }
  })
})

// Built with '<' + tag concatenation, not a literal `<script>`/`<style>` tag
// pair, so the named-template pre-transform's HTML-naive tokenizer (which
// runs on the raw file text, before real SFC parsing) can't mistake these
// injected-HTML tags for the surrounding <script setup> block's own tags.
const LINK_INTERCEPT_SCRIPT = `
${'<' + 'script>'}
document.addEventListener('click', function(e) {
  var a = e.target.closest('a');
  if (a && a.href && !a.href.startsWith('javascript')) {
    e.preventDefault();
    window.parent.postMessage({ type: 'open-url', url: a.href }, '*');
  }
});
${'<' + '/script>'}`

const LIGHT_BG_STYLE = `${'<' + 'style>'}
:root { color-scheme: light !important; }
html, body { background: #ffffff !important; color: #000000 !important; }
${'<' + '/style>'}`

// Built from parts so the raw source has no bare closing-head-tag substring —
// the named-template pre-transform tokenizes the whole file HTML-naively and
// mistakes that substring as a literal for a real closing tag.
const HEAD_CLOSE_TAG = '</' + 'head>'

const htmlBodyWithInterceptor = computed(() => {
  const html = auth.currentMessage.value?.html_body
  if (!html) return null
  const inject = LIGHT_BG_STYLE + LINK_INTERCEPT_SCRIPT
  return html.includes(HEAD_CLOSE_TAG)
    ? html.replace(HEAD_CLOSE_TAG, inject + HEAD_CLOSE_TAG)
    : inject + html
})


const showPatternDialog = ref(false)
const patternFrom = ref('')
const patternSubject = ref('')
const initialSubject = ref('')
const isSuggesting = ref(false)

const patternQuery = computed(() =>
  buildPatternQuery({ from: patternFrom.value, subject: patternSubject.value })
)
// Deletion targets the sender only — a subject phrase (even an LLM-suggested
// one) is too easy to get wrong and trash unrelated mail from the same sender.
const deleteQuery = computed(() => buildPatternQuery({ from: patternFrom.value }))

let suggestionToken = 0

/**
 * Open the "rule from this email" panel, seed sender/subject from the current
 * message, then refine the subject with a local-LLM suggestion (best-effort).
 */
async function openPatternDialog() {
  const msg = auth.currentMessage.value
  if (!msg) return
  auth.clearPatternFeedback()
  patternFrom.value = parseFromEmail(msg.from)
  patternSubject.value = (msg.subject ?? '').trim()
  initialSubject.value = patternSubject.value
  showPatternDialog.value = true
  isSuggesting.value = true
  const token = ++suggestionToken
  const suggestion = await pattern.suggestSubjectPattern(msg.subject)
  // Ignore the result if it was cancelled, the panel closed, or the user
  // edited the field while we were waiting.
  if (token !== suggestionToken) return
  if (showPatternDialog.value && patternSubject.value === initialSubject.value) {
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

const showActionLog = ref(false)
const showTemplates = ref(false)
const showFilters = ref(false)

async function trashByQueryAndClose() {
  const q = deleteQuery.value
  await auth.trashByQuery(q)
  if (!auth.trashQueryErrorKind.value) {
    showPatternDialog.value = false
    const count = auth.lastTrashedCount.value ?? 0
    $q.notify({
      message: `Переміщено в кошик: ${count}`,
      caption: q,
      color: 'positive',
      icon: 'sym_o_delete_sweep',
      timeout: 5000,
      position: 'top',
    })
  }
}
</script>

<template>
  <q-page class="column items-center q-pa-md" :class="{ 'has-bar': auth.isAuthenticated.value }">
    <template v-if="auth.isAuthenticated.value">
      <q-banner v-if="auth.inboxErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.inboxErrorKind.value) }}
      </q-banner>


      <template v-if="auth.currentMessage.value">
        <div class="row no-wrap q-col-gutter-md reader">
          <div class="col-12 col-md-6 column">
            <q-card flat bordered class="fit column">
              <q-card-section>
                <div class="text-overline text-grey-7">Оригінал</div>
                <div><strong>Від:</strong> {{ auth.currentMessage.value.from }}</div>
                <div><strong>Тема:</strong> {{ auth.currentMessage.value.subject }}</div>
                <div><strong>Дата:</strong> {{ auth.currentMessage.value.date }}</div>
              </q-card-section>
              <q-separator inset />
              <q-card-section v-if="auth.currentMessage.value.html_body" class="col message-html-section">
                <iframe
                  :srcdoc="htmlBodyWithInterceptor"
                  sandbox="allow-scripts"
                  class="message-iframe"
                  referrerpolicy="no-referrer" />
              </q-card-section>
              <q-card-section v-else class="message-body col">
                {{ auth.currentMessage.value.body }}
              </q-card-section>
            </q-card>
          </div>
          <div class="col-12 col-md-6 column">
            <q-card flat bordered class="fit column">
              <NewsletterView ref="newsletterViewRef" :message="auth.currentMessage.value" />
            </q-card>
          </div>
        </div>
      </template>
      <template v-else-if="auth.isMessageLoading.value">
        <q-card flat bordered style="max-width: 60ch; width: 100%">
          <q-card-section>
            <q-skeleton type="text" width="70%" />
            <q-skeleton type="text" width="60%" />
            <q-skeleton type="text" width="50%" />
          </q-card-section>
          <q-separator inset />
          <q-card-section>
            <q-skeleton type="text" />
            <q-skeleton type="text" />
            <q-skeleton type="text" width="80%" />
          </q-card-section>
        </q-card>
      </template>
      <q-banner v-else-if="auth.messageErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.messageErrorKind.value) }}
      </q-banner>
      <q-banner v-else class="bg-grey-2" rounded dense> Скринька порожня. </q-banner>

      <q-banner v-if="auth.saveErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.saveErrorKind.value) }}
      </q-banner>
      <q-banner v-if="auth.unsubscribeErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.unsubscribeErrorKind.value) }}
      </q-banner>
      <q-banner v-if="auth.trashErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.trashErrorKind.value) }}
      </q-banner>
    </template>

    <q-btn
      v-else
      @click="auth.login()"
      color="primary"
      icon-right="sym_o_login"
      size="md"
      :loading="auth.isLoading.value">
      <template v-if="!auth.isLoading.value">Увійти через Google</template>
      <template v-else>Зачекайте…</template>
    </q-btn>

    <q-banner v-if="auth.errorKind.value" class="bg-red-1 text-red-9" rounded dense>
      {{ errorMessage(auth.errorKind.value) }}
    </q-banner>

    <q-page-sticky v-if="auth.isAuthenticated.value" position="bottom" :offset="[0, 0]" expand>
      <q-toolbar class="bg-grey-2 text-primary action-bar q-px-md">
        <q-btn
          @click="auth.unsubscribeFromCurrent()"
          flat
          no-caps
          icon="sym_o_unsubscribe"
          label="Відписатися"
          :disable="!auth.currentMessage.value?.unsubscribe"
          :loading="auth.isUnsubscribing.value" />
        <q-btn
          @click="auth.saveCurrent()"
          flat
          no-caps
          icon="sym_o_bookmark_add"
          label="Зберегти"
          :disable="!auth.currentMessage.value"
          :loading="auth.isSaving.value" />
        <q-btn
          @click="newsletterViewRef?.flagAsTask()"
          flat
          no-caps
          icon="sym_o_task_alt"
          label="Задача"
          :disable="!auth.currentMessage.value" />
        <q-space />
        <q-btn
          @click="openPatternDialog()"
          flat
          no-caps
          icon="sym_o_rule"
          label="Правило"
          :disable="!auth.currentMessage.value" />
        <q-btn
          @click="auth.trashCurrent()"
          flat
          no-caps
          icon="sym_o_delete"
          label="Видалити"
          :disable="!auth.currentMessage.value"
          :loading="auth.isTrashing.value" />
        <q-btn
          @click="auth.loadRandomMessage()"
          flat
          no-caps
          icon="sym_o_skip_next"
          label="Показати інший"
          :loading="auth.isMessageLoading.value" />
        <q-btn
          @click="showActionLog = true"
          flat no-caps
          icon="sym_o_history"
          label="Журнал"
          :color="auth.actionLog.value.length ? 'primary' : undefined" />
        <q-btn
          @click="showTemplates = true"
          flat no-caps
          icon="sym_o_layers"
          label="Шаблони" />
        <q-btn
          @click="agentOpen = true"
          flat no-caps
          icon="sym_o_smart_toy"
          label="Агент"
          color="primary" />
        <q-btn
          @click="auditOpen = true"
          flat no-caps
          icon="sym_o_manage_search"
          title="Журнал запитів" />
        <q-btn flat no-caps round icon="sym_o_more_vert">
          <q-tooltip>Меню</q-tooltip>
          <q-menu anchor="top right" self="bottom right">
            <q-list style="min-width: 180px">
              <q-item clickable v-close-popup @click="showFilters = true">
                <q-item-section avatar><q-icon name="sym_o_filter_alt" /></q-item-section>
                <q-item-section>Фільтри Gmail</q-item-section>
              </q-item>
              <q-separator />
              <q-item clickable v-close-popup @click="auth.logout()">
                <q-item-section avatar><q-icon name="sym_o_logout" /></q-item-section>
                <q-item-section>Вийти</q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
      </q-toolbar>
    </q-page-sticky>

    <TemplatesManager v-model="showTemplates" />
    <GmailFiltersDialog v-model="showFilters" />

    <q-dialog v-model="showActionLog">
      <q-card style="min-width: 480px; max-width: 90vw">
        <q-card-section class="text-h6 row items-center">
          Журнал дій
          <q-space />
          <q-btn v-close-popup flat round dense icon="sym_o_close" />
        </q-card-section>
        <q-separator />
        <q-card-section style="max-height: 60vh; overflow-y: auto">
          <div v-if="!auth.actionLog.value.length" class="text-grey-6">Журнал порожній.</div>
          <q-list v-else separator>
            <q-item v-for="(entry, i) in auth.actionLog.value" :key="i" dense>
              <q-item-section>
                <q-item-label>{{ entry.text }}</q-item-label>
                <q-item-label caption>{{ new Date(entry.ts).toLocaleString('uk-UA') }}</q-item-label>
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </q-dialog>

    <q-dialog v-model="showPatternDialog">
      <q-card style="min-width: 360px; max-width: 90vw">
        <q-card-section class="text-h6">Правило для схожих листів</q-card-section>
        <q-card-section class="q-gutter-md">
          <q-input v-model="patternFrom" label="Від (email)" dense outlined clearable clear-icon="sym_o_close" />
          <q-input
            v-model="patternSubject"
            label="Тема містить"
            dense
            outlined
            :clearable="!isSuggesting"
            clear-icon="sym_o_close">
            <template v-if="isSuggesting" #append>
              <q-icon name="sym_o_close" class="cursor-pointer" @click="cancelSuggestion" />
            </template>
          </q-input>
          <div class="text-caption text-grey-7">
            Фільтр: <code>{{ patternQuery || '—' }}</code><br />
            Видалення (лише за відправником): <code>{{ deleteQuery || '—' }}</code>
          </div>
        </q-card-section>

        <q-card-section v-if="auth.trashQueryErrorKind.value || auth.filterErrorKind.value" class="q-pt-none">
          <q-banner class="bg-red-1 text-red-9" rounded dense>
            {{ errorMessage(auth.trashQueryErrorKind.value || auth.filterErrorKind.value) }}
          </q-banner>
        </q-card-section>
        <q-card-section v-else-if="auth.filterCreated.value" class="q-pt-none">
          <q-banner class="bg-green-1 text-green-9" rounded dense>
            Фільтр створено — нові такі листи йтимуть у кошик.
          </q-banner>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn v-close-popup flat no-caps label="Закрити" />
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
            @click="trashByQueryAndClose"
            flat
            no-caps
            color="negative"
            icon="sym_o_delete_sweep"
            label="Видалити всі такі"
            :disable="!deleteQuery"
            :loading="auth.isTrashingQuery.value" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <AgentDialog v-model="agentOpen" :agent="agent" />
    <AuditAnalysisDialog v-model="auditOpen" :agent="agent" />
  </q-page>
</template>

<style scoped>
.reader {
  width: 100%;
  align-self: stretch;
  flex: 1;
  min-height: 0;
}
.message-body,
.summary-body {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: inherit;
}
.message-html-section {
  padding: 0;
  overflow: hidden;
  flex: 1;
}
.message-iframe {
  width: 100%;
  height: 100%;
  min-height: 500px;
  border: none;
  display: block;
}
.has-bar {
  padding-bottom: calc(64px + env(safe-area-inset-bottom));
}
.action-bar {
  border-top: 1px solid rgba(0, 0, 0, 0.12);
  padding-bottom: env(safe-area-inset-bottom);
}
</style>
