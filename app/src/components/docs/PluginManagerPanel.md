---
type: Vue Component
title: PluginManagerPanel.vue
resource: app/src/components/PluginManagerPanel.vue
docgen:
  crc: faa5f6f4
---

## Огляд

Панель керування встановленими плагінами: список, disable, uninstall, install з consent.

## Поведінка

Відкривається з Login через `v-model`; викликає Tauri-команди Plugin Manager і показує PluginConsentDialog за потреби.
