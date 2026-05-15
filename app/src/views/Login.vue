<script setup>
import { onMounted } from 'vue'
import { useAuthStore } from '../services/auth-store.js'
import { errorMessage } from '../i18n/auth-errors.js'

const auth = useAuthStore()
onMounted(() => auth.initialize())
</script>

<template>
  <main class="login">
    <h1>MLMaiL</h1>
    <div v-if="auth.isAuthenticated.value" class="signed-in">
      <p>Ви увійшли як {{ auth.email.value }}</p>
      <p v-if="auth.inboxCount.value !== null" class="inbox-count">
        Листів у скриньці: {{ auth.inboxCount.value }}
      </p>
      <p v-else-if="auth.inboxErrorKind.value" class="error">
        {{ errorMessage(auth.inboxErrorKind.value) }}
      </p>
      <p v-else class="inbox-count muted">Листів у скриньці: …</p>
      <section v-if="auth.currentMessage.value" class="message">
        <header class="message-head">
          <p><strong>Від:</strong> {{ auth.currentMessage.value.from }}</p>
          <p><strong>Тема:</strong> {{ auth.currentMessage.value.subject }}</p>
          <p><strong>Дата:</strong> {{ auth.currentMessage.value.date }}</p>
        </header>
        <pre class="message-body">{{ auth.currentMessage.value.body }}</pre>
      </section>
      <p v-else-if="auth.isMessageLoading.value" class="muted">Завантаження…</p>
      <p v-else-if="auth.messageErrorKind.value" class="error">
        {{ errorMessage(auth.messageErrorKind.value) }}
      </p>
      <button
        @click="auth.loadRandomMessage()"
        type="button"
        :disabled="auth.isMessageLoading.value"
      >
        Показати інший
      </button>
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

.inbox-count {
  margin: 0;
}

.inbox-count.muted {
  opacity: 0.6;
}

.error {
  color: #b00020;
}

.message {
  max-width: 60ch;
  text-align: left;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 0.8em 1em;
  background: rgb(0 0 0 / 2%);
}

.message-head p {
  margin: 0.1em 0;
}

.message-body {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  margin: 0.6em 0 0;
  font-family: inherit;
}

.muted {
  opacity: 0.6;
}
</style>
