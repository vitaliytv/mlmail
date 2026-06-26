import { onMounted } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { useQuasar } from 'quasar'

export function useUpdater() {
  const $q = useQuasar()

  async function checkForUpdates() {
    try {
      const update = await check()
      if (!update) return

      $q.dialog({
        title: 'Доступне оновлення',
        message: `Версія ${update.version} готова. Встановити зараз?`,
        cancel: { label: 'Пізніше', flat: true },
        ok: { label: 'Встановити', color: 'primary' },
        persistent: true,
      }).onOk(async () => {
        let downloaded = 0
        let total = 0
        const dismiss = $q.notify({ group: false, timeout: 0, spinner: true, message: 'Завантаження оновлення…' })
        try {
          await update.downloadAndInstall(event => {
            if (event.event === 'Started') total = event.data.contentLength ?? 0
            if (event.event === 'Progress') {
              downloaded += event.data.chunkLength
              if (total) dismiss({ message: `Завантаження… ${Math.round((downloaded / total) * 100)}%` })
            }
            if (event.event === 'Finished') dismiss()
          })
          $q.notify({ message: 'Оновлення встановлено. Перезапусти програму.', color: 'positive', timeout: 0, actions: [{ label: 'OK', color: 'white' }] })
        } catch (e) {
          dismiss()
          $q.notify({ message: `Помилка оновлення: ${e}`, color: 'negative', timeout: 5000 })
        }
      })
    } catch {
      // Нема мережі, Android, або updater не налаштовано — мовчки ігноруємо
    }
  }

  onMounted(() => {
    setTimeout(checkForUpdates, 3000)
  })
}
