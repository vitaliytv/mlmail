<script setup>
// @ts-nocheck

import LoginView from './views/Login.vue';
import { useAuthStore } from './services/auth-store.js';
const auth = useAuthStore();

</script>

<template>
  <q-layout view="hHh lpR fFf">
    <q-page-container>
      <LoginView />
    </q-page-container>

    <q-footer v-if="auth.isAuthenticated.value" elevated class="bg-white text-dark">
      <q-toolbar class="justify-center q-gutter-sm">
        <q-btn
          @click="auth.loadRandomMessage()"
          color="primary"
          icon="sym_o_refresh"
          :loading="auth.isMessageLoading.value">
          Показати інший
        </q-btn>
        <q-btn @click="auth.logout()" flat color="grey-8" icon="sym_o_logout">Вийти</q-btn>
      </q-toolbar>
    </q-footer>
  </q-layout>
</template>
