---
type: Vue Component
title: PluginManagerPanel.vue
resource: app/src/components/PluginManagerPanel.vue
docgen:
  crc: 295cb99b
---

## Огляд

Панель Plugin Manager: список, signed sample, install з абсолютного шляху до `.n-plugin`, disable/uninstall, consent-діалог.

## Поведінка

«Встановити signed sample» → `plugin_manager_install_sample`. «Встановити з файлу» → previewPath/confirmInstall з `allow_unsigned: false` і companion `.pub`.
