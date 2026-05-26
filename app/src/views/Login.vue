<script setup>
// @ts-nocheck

function stryNS_9fa48() {
  var g = typeof globalThis === 'object' && globalThis && globalThis.Math === Math && globalThis || new Function("return this")();
  var ns = g.__stryker__ || (g.__stryker__ = {});
  if (ns.activeMutant === undefined && g.process && g.process.env && g.process.env.__STRYKER_ACTIVE_MUTANT__) {
    ns.activeMutant = g.process.env.__STRYKER_ACTIVE_MUTANT__;
  }
  function retrieveNS() {
    return ns;
  }
  stryNS_9fa48 = retrieveNS;
  return retrieveNS();
}
stryNS_9fa48();
function stryCov_9fa48() {
  var ns = stryNS_9fa48();
  var cov = ns.mutantCoverage || (ns.mutantCoverage = {
    static: {},
    perTest: {}
  });
  function cover() {
    var c = cov.static;
    if (ns.currentTestId) {
      c = cov.perTest[ns.currentTestId] = cov.perTest[ns.currentTestId] || {};
    }
    var a = arguments;
    for (var i = 0; i < a.length; i++) {
      c[a[i]] = (c[a[i]] || 0) + 1;
    }
  }
  stryCov_9fa48 = cover;
  cover.apply(null, arguments);
}
function stryMutAct_9fa48(id) {
  var ns = stryNS_9fa48();
  function isActive(id) {
    if (ns.activeMutant === id) {
      if (ns.hitCount !== void 0 && ++ns.hitCount > ns.hitLimit) {
        throw new Error('Stryker: Hit count limit reached (' + ns.hitCount + ')');
      }
      return true;
    }
    return false;
  }
  stryMutAct_9fa48 = isActive;
  return isActive(id);
}
import { errorMessage } from '../i18n/auth-errors.js';
import { loginMessages } from '../i18n/login.js';
import { useAuthStore } from '../services/auth-store.js';
const auth = useAuthStore();
onMounted(stryMutAct_9fa48("103") ? () => undefined : (stryCov_9fa48("103"), () => auth.initialize()));

/**
 * @param {boolean} value whether to request only newsletters
 */
function toggleOnlyNewsletters(value) {
  if (stryMutAct_9fa48("104")) {
    {}
  } else {
    stryCov_9fa48("104");
    auth.setOnlyNewsletters(value);
    auth.loadRandomMessage();
  }
}

</script>

<template>
  <q-page class="column items-center q-gutter-md q-pa-md" :class="{ 'has-bar': auth.isAuthenticated.value }">
    <div class="text-h4">{{ loginMessages.appTitle }}</div>

    <template v-if="auth.isAuthenticated.value">
      <div class="text-body1">Ви увійшли як {{ auth.email.value }}</div>

      <q-chip v-if="auth.inboxCount.value !== null" icon="sym_o_mail" color="primary" text-color="white">
        Листів у скриньці: {{ auth.inboxCount.value }}
      </q-chip>
      <q-banner v-else-if="auth.inboxErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.inboxErrorKind.value) }}
      </q-banner>
      <q-skeleton v-else type="QChip" width="180px" />

      <q-toggle
        @update:model-value="toggleOnlyNewsletters"
        :model-value="auth.onlyNewsletters.value"
        label="Тільки newsletters"
        color="primary"
        dense />

      <template v-if="auth.currentMessage.value">
        <q-card flat bordered style="max-width: 60ch; width: 100%">
          <q-card-section>
            <div><strong>Від:</strong> {{ auth.currentMessage.value.from }}</div>
            <div><strong>Тема:</strong> {{ auth.currentMessage.value.subject }}</div>
            <div><strong>Дата:</strong> {{ auth.currentMessage.value.date }}</div>
          </q-card-section>
          <q-separator inset />
          <q-card-section class="message-body">
            {{ auth.currentMessage.value.body }}
          </q-card-section>
        </q-card>
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

      <q-banner v-if="auth.unsubscribeErrorKind.value" class="bg-red-1 text-red-9" rounded dense>
        {{ errorMessage(auth.unsubscribeErrorKind.value) }}
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
        <q-space />
        <q-btn
          @click="auth.loadRandomMessage()"
          flat
          no-caps
          icon="sym_o_skip_next"
          label="Показати інший"
          :loading="auth.isMessageLoading.value" />
        <q-btn @click="auth.logout()" flat no-caps icon="sym_o_logout" label="Вийти" />
      </q-toolbar>
    </q-page-sticky>
  </q-page>
</template>

<style scoped>
.message-body {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: inherit;
}
.has-bar {
  padding-bottom: calc(64px + env(safe-area-inset-bottom));
}
.action-bar {
  border-top: 1px solid rgba(0, 0, 0, 0.12);
  padding-bottom: env(safe-area-inset-bottom);
}
</style>
