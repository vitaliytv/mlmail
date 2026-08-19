# Durable context плагінів у mlmail

Інструкції для авторів нових Components, supported contracts і локальна E2E matrix описані в
[посібнику з розробки плагінів](./plugin-authoring.md).

## Призначення

`mlmail` використовує `n-plugin-runtime` як durable lifecycle coordinator між
product-owned Gmail capability та встановленим WebAssembly Component. Це не
переносить Gmail token у plugin runtime: OAuth, HTTP endpoint і Gmail response
залишаються всередині Rust host adapter.

Поточний context містить дві окремі exact-release instances:

```text
plugin:draft-helper
  requires nitra:gmail/drafts@0.1.0
            │
            ▼
mlmail:gmail-provider
  provides nitra:gmail/drafts@0.1.0
```

`mlmail:gmail-provider` — native product-owned typed provider, а не
встановлюваний plugin. `plugin:draft-helper` — встановлений WebAssembly
Component. Однаковий WIT identity є dependency key; довільних JSON routes або
string ABI між ними немає.

## Durable storage

Application data містить:

```text
n-plugin/
├── registry.sqlite3   activation generations і edge policy
├── context.sqlite3    desired context, instance lifecycle та repair events
├── installed.json     UI projection встановлених root plugins
└── cas/               immutable Component bytes
```

`context.sqlite3` зберігає повний desired context як copy-on-write generations.
Manual disable залишає Draft Helper у desired generation з `enabled = false`.
Uninstall прибирає його entry, але не змінює identity native Gmail provider.

## Lifecycle операцій

### Install або update

1. Installer перевіряє Component manifest і typed trigger.
2. Activation registry публікує immutable Component generation.
3. `mlmail` публікує новий desired context із Gmail provider та exact Draft
   Helper release.
4. Наступна activation спочатку вводить provider у service, потім consumer.

Зміна exact Gmail provider release або digest викликає dependency-first
replacement: Draft Helper припиняє приймати нову роботу, старий provider
вивантажується, новий provider активується, після чого Draft Helper
перезавантажується з новим committed provider view.

### Disable і uninstall

- Manual disable не видаляє exact Draft Helper release з desired state та не
  може бути автоматично скасований появою dependency.
- Re-enable знову активує provider перед Draft Helper.
- Uninstall видаляє Draft Helper з наступної complete desired generation.

### Create draft

Перед отриманням OAuth token і Gmail `POST /drafts` команда:

1. відновлює interrupted lifecycle попереднього process;
2. offline відтворює останню committed desired generation;
3. перевіряє, що обидві instances мають стан `Active`;
4. лише після цього виконує typed Draft Helper call.

Створення Gmail draft є зовнішньою emission. Воно навмисно не реєструється як
revertible lifecycle effect і не повторюється автоматично. Якщо cleanup context
після успішної Gmail відповіді завершується помилкою, команда все одно повертає
успішний `draft_id`, а cleanup error пишеться в log. Це не провокує UI повторити
вже виконану emission і створити duplicate draft.

## Cold start і offline режим

Cold start не виконує package resolution і не потребує registry або Gmail
network. Runtime читає exact desired generation з `context.sqlite3`, ремонтує
interrupted instances у порядку consumers → providers і повторно активує їх у
порядку providers → consumers. Gmail network потрібна лише для explicit user
action після settled activation.

Якщо restart repair не може підтвердити safe inactive boundary, runtime працює
fail-closed: новий Component call не запускається. У durable audit не
записуються OAuth token, Gmail payload, recipient, subject або body.

## Інтеграційний контракт

Новий product provider додається як окрема stable instance з:

- exact `ReleaseIdentity`;
- versioned WIT interfaces у `provides`;
- generated typed Rust binding;
- product-owned restart repair для ресурсів, які можуть пережити process;
- без sensitive values у desired context або lifecycle events.

Online side effects повинні виконуватися тільки через coordinator gate після
settled activation. Reversible host acquisitions реєструються у
`RevertibleEffectScope`; emissions, які host не може відкочувати ексклюзивно,
ніколи не маскуються під acquisitions.

## Перевірки

Product tests підтверджують:

- provider-first activation і consumer-first withdrawal;
- dependent reload під час provider replacement;
- блокування emission до active state обох instances;
- offline replay та dependency-first repair після interrupted process.

Запуск:

```bash
cargo test -p mlmail plugin_context --lib
```
