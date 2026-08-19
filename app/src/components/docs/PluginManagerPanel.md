---
type: Vue Component
title: PluginManagerPanel.vue
resource: app/src/components/PluginManagerPanel.vue
docgen:
  crc: 088433b4
  model: manual
---

## Огляд

General-purpose Plugin Manager попередньо перевіряє typed WebAssembly Components, збирає
opaque consent decisions і викликає лише product-owned typed actions для exact release,
повернутого backend projection.

## Поведінка

Native picker передає шлях у `plugin_manager_preflight`. Compatibility result, exact
package/version/digest, product actions, dependencies і account-bound capability rows
відображаються до activation. Cancel не викликає confirm, а confirm повертає backend лише
`previewId`, `expectedRelease` та пари `requirementId`/`allow`, після чого перезавантажує committed
projection.

Installed rows показують exact release, activation generation, enabled і lifecycle state та
product-owned action descriptors. Draft Helper і Booking Finder мають окремі typed Tauri commands;
dynamic command names не використовуються. Action, disable та uninstall завжди отримують exact
`ReleaseIdentity` вибраного рядка. Loading ізольований на рівні рядка, а operation error не
стирається успішним reload.

## Інтерфейс компонента

- `modelValue` керує видимістю діалогу.
- `update:modelValue` синхронізує закриття або відкриття з батьківським компонентом.

## Гарантії поведінки

- Incompatible Component не активується з preview UI.
- UI не формує grant keys і не передає capability, account або scope назад у confirm command.
- Невідомий action kind не перетворюється на dynamic Tauri invocation.
- Кожна mutation та typed action адресує exact package/version/digest.
