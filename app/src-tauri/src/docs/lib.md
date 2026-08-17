---
type: Module
title: lib.rs
resource: app/src-tauri/src/lib.rs
docgen:
  crc: ae38abdb
  model: manual
---

## Огляд

Точка входу Tauri: opener, dialog, http, agent, window-state/updater (desktop); реєстр команд auth/gmail/newsletter/llm/plugins і durable context coordinator для plugin instances. Реєструє явні команди конфігурації та викликів локальної OpenAI-compatible LLM.
