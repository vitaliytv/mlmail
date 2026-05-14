<script setup>
import { onMounted } from 'vue'
import { useAuthStore } from '../services/auth-store.js'
import { useMailboxStore } from '../services/mailbox-store.js'
import { errorMessage, inboxCountLabel } from '../i18n/auth-errors.js'

const auth = useAuthStore()
const mailbox = useMailboxStore()

onMounted(async () => {
  await auth.initialize()
  if (auth.isAuthenticated.value) {
    mailbox.fetchInboxCount()
  }
})
</script>

<template>
  <main class="login">
    <h1>MLMaiL</h1>
    <div v-if="auth.isAuthenticated.value" class="signed-in">
      <p>Ви увійшли як {{ auth.email.value }}</p>
      <p v-if="mailbox.isLoading.value" class="inbox-count">…</p>
      <p v-else-if="mailbox.errorKind.value" class="inbox-count error">
        {{ errorMessage(mailbox.errorKind.value) }}
      </p>
      <p v-else-if="mailbox.inboxCount.value !== null" class="inbox-count">
        {{ inboxCountLabel(mailbox.inboxCount.value) }}
      </p>
      <button @click="auth.logout()" type="button">Вийти</button>
    </div>
    <button
      v-else
      @click="auth.login()"
      type="button"
      :disabled="auth.isLoading.value"
    >
      {{ auth.isLoading.value ? 'Зачекайте…' : 'Увійти через Google' }}
    </button>
    <p v-if="auth.errorKind.value" class="error">
      {{ errorMessage(auth.errorKind.value) }}
    </p>
  </main>
</template>

<style scoped>
.login {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding-top: 10vh;
  text-align: center;
}

.login button {
  padding: 0.6em 1.2em;
  border-radius: 8px;
  border: 1px solid transparent;
  background: #fff;
  cursor: pointer;
  font: inherit;
}

.login button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.error {
  color: #b00020;
}
</style>
