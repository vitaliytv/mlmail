---
type: Rust Module
title: plugin_consent.rs
resource: app/src-tauri/src/gmail/plugin_consent.rs
docgen:
  crc: 11a17162
  model: manual
---

## Огляд

Надає Gmail-specific typed adapter над canonical product grant store. Adapter зіставляє exact
`nitra:gmail/search@0.1.0` з `mail:search` для конкретного Component release та account identity.

## Поведінка

`GmailSearchConsentStore` зберігає й перевіряє той самий exact `PluginGrantKey`, який створює
generic installation confirm. Legacy exact records можуть бути перепаковані у canonical grant
format, але не розширюються на інший digest, account або host edge.

## Публічний API

- `GmailSearchConsentStore` — typed grant/require adapter для Gmail search.
- `GmailSearchConsent` — legacy exact record, що підтримує безпечне перепакування.
- `MAIL_SEARCH_CAPABILITY` — stable product capability `mail:search`.
- `consent_store_path` — canonical `n-plugin/grants.json` path.

## Гарантії поведінки

- Відсутній exact grant повертає payload-free `grant-required` error.
- OAuth token, query і message data не записуються у consent storage.
