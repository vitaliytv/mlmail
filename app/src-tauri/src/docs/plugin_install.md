---
type: Rust Module
title: plugin_install.rs
resource: app/src-tauri/src/plugin_install.rs
docgen:
  crc: 4376d4a1
  model: manual
---

## Огляд

Формує read-only installation preview для manifest-bearing WebAssembly Component. Preview
показує exact release, підтримані typed triggers та actions, required dependencies і capability
consents без tokens, mail data або інших sensitive values.

## Поведінка

Preflight перевіряє Component format та embedded manifest, зіставляє entrypoints, triggers і host
imports із product-owned registry, а dependency-free candidate пропускає через той самий
`ActivationCompiler`, що використовується для activation. Unknown contract або unsupported
entrypoint повертає incompatible preview з причиною. Required dependency declarations
відображаються у preview, але лишають candidate incompatible до graph resolver task.

Preflight будує singleton graph лише в пам’яті. Account-aware варіант додає до preview exact
public account identity, sealed grant scope, opaque requirement identifiers та `preview_id`, що
зв’язує Component bytes, contract fingerprint і весь consent set. Він не відкриває activation
registry, не записує CAS, SQLite, installed projection, context або `.n-plugin.lock`.

## Публічний API

- `PluginInstallPreview` — serializable public projection exact release та compatibility result.
- `PluginActionPreview` — product action label і exact typed trigger.
- `PluginDependencyPreview` — logical dependency edge, package, requirement та typed imports.
- `PluginCapabilityPreview` — capability, exact host interface та consent scope.
- `PluginCapabilityAccountScope` — account-scoped або application-scoped consent.
- `preflight_component` — pure validation і dry composition для одного Component.
- `preflight_component_for_account` — формує authoritative account-bound consent preview.

## Гарантії поведінки

- Package name або publisher не визначає підтримку plugin.
- Capability requirements походять лише з typed imports, зареєстрованих application registry.
- Незареєстровані WASI imports fail closed так само, як інші unknown host interfaces.
- Compatibility mismatch не створює persistent plugin state.
- Dependency graph не активується частково.
- Відсутній account identity ніколи не розширює account-scoped requirement до application scope.
