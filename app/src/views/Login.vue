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
      <button type="button" @click="auth.logout()">Вийти</button>
    </div>
    <button
      v-else
      type="button"
      :disabled="auth.isLoading.value"
      @click="auth.login()"
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
