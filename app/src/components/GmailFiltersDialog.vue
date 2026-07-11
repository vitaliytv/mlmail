<script setup>
import { useQuasar } from 'quasar'
import { errorMessage } from '../i18n/auth-errors.js'
import { useAuthStore } from '../services/auth-store.js'

const show = defineModel({ default: false })
const auth = useAuthStore()
const $q = useQuasar()

watch(show, v => {
  if (v) auth.listFilters()
})

/**
 *
 * @param f
 */
function criteriaLabel(f) {
  const parts = []
  if (f.criteria?.from) parts.push(`від: ${f.criteria.from}`)
  if (f.criteria?.subject) parts.push(`тема: ${f.criteria.subject}`)
  return parts.length ? parts.join(' · ') : '(без умов)'
}

/**
 *
 * @param f
 */
function confirmDelete(f) {
  $q.dialog({
    title: 'Видалити фільтр?',
    message: `Фільтр "${criteriaLabel(f)}" буде видалено остаточно.`,
    cancel: true,
    persistent: true,
    ok: { label: 'Видалити', color: 'negative', flat: true },
    cancel: { label: 'Скасувати', flat: true }
  }).onOk(() => auth.deleteFilter(f.id))
}
</script>

<template>
  <q-dialog v-model="show" maximized>
    <q-card>
      <q-bar>
        <span class="text-weight-medium">Фільтри Gmail</span>
        <q-space />
        <q-btn v-close-popup flat round dense icon="sym_o_close" />
      </q-bar>

      <q-card-section>
        <q-banner v-if="auth.filtersErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
          {{ errorMessage(auth.filtersErrorKind.value) }}
        </q-banner>
        <q-banner v-if="auth.deleteFilterErrorKind.value" class="bg-red-1 text-red-9 q-mt-sm" rounded dense>
          {{ errorMessage(auth.deleteFilterErrorKind.value) }}
        </q-banner>

        <div v-if="auth.isLoadingFilters.value" class="text-grey-6 q-pa-md text-center">
          <q-spinner color="primary" size="24px" />
        </div>
        <div v-else-if="!auth.filters.value.length && !auth.filtersErrorKind.value" class="text-grey-6">
          Фільтрів ще немає.
        </div>
        <q-list v-else bordered separator rounded>
          <q-item v-for="f in auth.filters.value" :key="f.id">
            <q-item-section>
              <q-item-label>{{ criteriaLabel(f) }}</q-item-label>
              <q-item-label caption>{{ f.id }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-btn
                @click="confirmDelete(f)"
                flat
                dense
                round
                icon="sym_o_delete"
                color="negative"
                :loading="auth.isDeletingFilterId.value === f.id"
                :disable="!!auth.isDeletingFilterId.value" />
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>
