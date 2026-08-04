---
type: Vue Component
title: PluginManagerPanel.vue
resource: app/src/components/PluginManagerPanel.vue
docgen:
  crc: 832ce1ac
---

## Огляд

Панель Plugin Manager: список, signed sample, native picker `.n-plugin` (`@tauri-apps/plugin-dialog`), disable/uninstall, consent-діалог.

## Поведінка

«Встановити signed sample» → `plugin_manager_install_sample`. «Обрати .n-plugin» → `open()` + previewPath/confirmInstall з `allow_unsigned: false` і companion `.pub`.
