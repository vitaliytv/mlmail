---
type: Module
title: manager.rs
resource: app/src-tauri/mlmail-plugin-host/src/manager.rs
docgen:
  crc: d994307a
---

## Огляд

Plugin Manager: install з consent-diff, disable, uninstall з purge grants/audit.

## Публічний API

- `install_plugin` — розпаковка пакета, consent за escalation, збереження grants
- `set_enabled` — увімкнути/вимкнути встановлений плагін
- `uninstall_plugin` — видалення пакета й purge повʼязаних даних
- `list_installed` / `consent_diff` — стан і порівняння capabilities
