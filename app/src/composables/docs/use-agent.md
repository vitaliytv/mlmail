---
type: JS Module
title: use-agent.js
resource: app/src/composables/use-agent.js
docgen:
  crc: 37c968a1
---

## Огляд

Обгортка над `useAcpAgent` з `@7n/tauri-components/vue` для mlmail: підставляє доменний каталог `TOOLS` і робочий каталог `homeDir()` (або `"."` поза Tauri).

## Поведінка

1. На імпорті резолвить `cwd` через `homeDir()`; при помилці лишає `"."`.
2. `useAgent()` повертає gateway з `useAcpAgent({ catalog: TOOLS, cwd })`.
3. Види агентів і model tiers беруться з бекенду (`acp_list_tiers`) — frontend spawn-пресети не передаються.

## Гарантії поведінки

- Не містить локальних agent presets (`CODEX_ACP_AGENT_PRESET` тощо).
- Не пише у ФС/БД самостійно — лише делегує в `useAcpAgent`.
