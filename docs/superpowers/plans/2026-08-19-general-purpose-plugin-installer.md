# General-purpose plugin installer для mlmail — implementation plan

**Статус: реалізацію Tasks 1–8 завершено 2026-08-19.** Local deterministic E2E
matrix пройшла повністю; opt-in OCI smoke лишається окремим network gate з pinned
public fixture. Повний `test:rust` wrapper у цьому runner обмежений вільним місцем під
час linking desktop binary, тоді як `cargo test --lib` (168 tests), real Component
matrix і Vue suite пройшли.

> **Для agentic workers:** реалізовувати послідовно, по одному task на PR. Кожен task
> завершується власними тестами, `n-doc-files`, delta-lint і change-файлом. Не переносити
> domain contracts назад у `n-plugin`.

**Мета:** перетворити Draft Helper-specific Plugin Manager на product-local typed
installer і dispatcher, через які нові `.n-plugin` Components із уже підтриманими
mlmail WIT contracts встановлюються без перекомпіляції застосунку.

**Архітектура:** `n-plugin` лишається domain-agnostic і відповідає за Component
inspection, graph resolution, compatibility, activation generations, CAS та lifecycle.
`mlmail` володіє статичним typed registry своїх WIT interfaces і triggers, consent
mapping та invocation adapters. Installer перевіряє Component проти цього registry,
але не містить окремих `validate_draft_helper`/`install_draft_helper` гілок. Виклики
лишаються typed: універсального JSON/string ABI або dynamic broker не додаємо.

**Поточна база:** `mlmail app@0.28.0`, `n-plugin v0.2.0`, PR #41-#44. Booking Finder
є першим незалежним Component із `mail:search` consent, а Draft Helper — першим
Component, встановленим через UI та durable context.

**Специфікації:**

- [`docs/specs/2026-08-07-domain-agnostic-plugin-runtime.md`](../../specs/2026-08-07-domain-agnostic-plugin-runtime.md)
- [`docs/specs/2026-08-02-wasm-a2ui-plugin-platform.md`](../../specs/2026-08-02-wasm-a2ui-plugin-platform.md)

---

## Межа готовності

Після реалізації має виконуватися така матриця:

| Сценарій | Очікувана поведінка |
| --- | --- |
| Новий Component використовує вже зареєстрований trigger/import | Встановлюється без нового build `mlmail` |
| Новий Component додає новий domain WIT contract | Потребує product-local зміни `mlmail`, але не `n-plugin` |
| Два Components реалізують один trigger | Обидва встановлюються; користувач явно обирає exact release для виклику |
| Component має required dependencies | Graph резолвиться за exact lock і активується атомарно |
| Dependency не імпортує host capability | Не створює додаткового consent |
| Component просить невідомий import або trigger | Installation preflight відхиляє його до activation |
| Offline restart | Active generation відновлюється з CAS/SQLite без мережі |

## Не входить у цей план

- plugin catalog або marketplace;
- signatures, publisher key transfer і supply-chain verification;
- resource limits;
- optional dependencies або SemVer ranges для raw Components;
- підтримка legacy core-Wasm, JSON/string ABI чи `mlmail-plugin-host`;
- runtime-завантаження нового WIT contract без product update;
- authoring languages, крім Rust у поточному M0 toolchain;
- підтримка старих Vue projections.

---

## Task 1: Product-owned typed contract registry

**Файли:**

- Create: `app/src-tauri/src/plugin_contracts.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/src/gmail/plugin_runtime.rs`
- Test: unit tests у `app/src-tauri/src/plugin_contracts.rs`
- Generated docs: `app/src-tauri/src/docs/plugin_contracts.md`

- [ ] **Step 1: Зафіксувати failing registry tests**

Покрити щонайменше:

- registry містить exact Draft Helper і Booking Finder trigger identities;
- host inventory містить `nitra:gmail/search@0.1.0` і
  `nitra:gmail/drafts@0.1.0` з digest із того самого `wkg.lock`;
- невідомий trigger не резолвиться;
- registry не дозволяє дублікати trigger identity;
- public environment будується з того самого набору registrations, що linker.

- [ ] **Step 2: Додати `MlmailPluginContractRegistry`**

Registry має бути product-owned і статично typed. Він повертає:

- `HostInterfaceInventory` для activation compiler;
- `ApplicationTriggerInventory` для installation preflight;
- product action kind для кожного відомого trigger;
- capability requirements для consent preview;
- generated linker registrations через наявний `PluginHostInterfaceRegistry`.

Не зберігати closures або domain enum у `n-plugin`. Не створювати другий package
resolver: exact WIT descriptors і надалі читаються з `app/src-tauri/wkg.lock`.

- [ ] **Step 3: Перевести `build_gmail_plugin_runtime` на registry**

Прибрати паралельне ручне складання host interfaces і triggers. Runtime environment,
application identity та compatibility metadata мають походити з одного registry.

- [ ] **Step 4: Перевірити task**

Run: `bun run --cwd app test:rust`

Expected: registry tests зелені; існуючі Gmail runtime tests не змінюють поведінку.

---

## Task 2: Generic installation preflight

**Файли:**

- Create: `app/src-tauri/src/plugin_install.rs`
- Modify: `app/src-tauri/src/plugins.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Test: unit tests у `app/src-tauri/src/plugin_install.rs` і `plugins.rs`
- Generated docs: `app/src-tauri/src/docs/plugin_install.md`,
  `app/src-tauri/src/docs/plugins.md`

- [ ] **Step 1: Зафіксувати failing preflight tests**

Сценарії:

- Draft Helper і Booking Finder проходять один installer path;
- Component із невідомим trigger відхиляється;
- Component із невідомим host import відхиляється;
- core-Wasm і Component без `nitra.plugin-manifest/v1` відхиляються;
- Component із нульовою кількістю підтриманих entrypoints відхиляється;
- preflight нічого не записує у CAS, SQLite чи installed projection;
- результат містить exact release, triggers, dependency summary і required consents.

- [ ] **Step 2: Ввести `PluginInstallPreview`**

Preview є серіалізованою product projection, а не новим manifest format. Він містить
лише public/non-sensitive дані, необхідні UI для рішення користувача:

- exact `ReleaseIdentity`;
- підтримані triggers і product action labels;
- required capabilities та account scope без tokens/query/mail bodies;
- required dependency identities;
- compatibility result і причину відмови.

- [ ] **Step 3: Замінити Draft Helper-specific validation**

Прибрати `validate_draft_helper` як installation boundary. Generic validation має:

1. перевірити valid Component та embedded manifest;
2. зіставити imports/triggers із `MlmailPluginContractRegistry`;
3. виконати dry graph composition через `ActivationCompiler`;
4. повернути preview без activation side effects.

Перший slice може лишати dependency graph activation для Task 6, але не повинен
відхиляти manifest лише через назву package або через те, що він не Draft Helper.

- [ ] **Step 4: Додати Tauri preflight command**

Додати `plugin_manager_preflight(path) -> PluginInstallPreview`. Існуючий
`plugin_manager_install` тимчасово лишити compatibility wrapper до Task 3, але весь
validation flow уже має проходити через preflight service.

- [ ] **Step 5: Перевірити task**

Run: `bun run --cwd app test:rust`

Expected: обидва bundled Components проходять той самий preflight; invalid fixtures
відхиляються до запису state.

---

## Task 3: Consent-aware generic activation

**Файли:**

- Create: `app/src-tauri/src/plugin_grants.rs`
- Modify: `app/src-tauri/src/plugin_install.rs`
- Modify: `app/src-tauri/src/plugins.rs`
- Modify: `app/src-tauri/src/gmail/plugin_consent.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Test: Rust unit/integration tests для preview, grant і activation rollback
- Generated docs: відповідні `app/src-tauri/src/docs/*.md`

- [ ] **Step 1: Узагальнити product-local grant store**

Зберігати grant за exact Component release, capability, account identity і edge scope.
Не переносити OAuth token, Gmail query чи mail content у grant/audit records.

Міграція наявного `GmailSearchConsentStore` має зберегти deny-by-default для
`mail:search`. Draft capability також має бути виражена через product mapping, а не
через package-name special case.

- [ ] **Step 2: Розділити preview і confirm**

Додати явний command на підтвердження installation preview. Confirm приймає exact
preview/release identity та user grant decisions, повторно перевіряє Component bytes і
лише після цього публікує activation generation.

Застарілий preview, інший digest або ширший capability set повинні вимагати нового
preview/consent, а не мовчки продовжувати installation.

- [ ] **Step 3: Гарантувати atomic activation**

Failed consent persistence, composition, CAS write або index write не змінює active
generation. Успішний confirm оновлює activation registry, installed projection і durable
context як одну логічну операцію з recoverable reconciliation.

- [ ] **Step 4: Перевірити task**

Run: `bun run --cwd app test:rust`

Expected: Booking Finder denied без `mail:search`, exact grant дозволяє activation,
інший digest/account лишається denied; Draft Helper regression зелений.

---

## Task 4: Exact plugin selection для typed invocation

**Файли:**

- Create: `app/src-tauri/src/plugin_dispatch.rs`
- Modify: `app/src-tauri/src/plugins.rs`
- Modify: `app/src-tauri/src/gmail/plugin_draft_helper.rs`
- Modify: `app/src-tauri/src/gmail/plugin_booking_finder.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Test: Rust tests для двох packages з однаковим trigger
- Generated docs: відповідні `app/src-tauri/src/docs/*.md` і Gmail file docs

- [ ] **Step 1: Зафіксувати failing multi-plugin test**

Встановити два Components з однаковим Draft Helper trigger, викликати кожен за exact
release і довести, що dispatcher не обирає перший package зі sorted index.

- [ ] **Step 2: Додати explicit invocation target**

Typed commands повинні приймати exact release target або package + digest. Package без
digest недостатній через update race. Dispatcher перевіряє installed/enabled lifecycle і
active generation саме цього release.

- [ ] **Step 3: Зберегти typed command surface**

Не додавати `plugin_invoke(name, json)`. Лишити окремі product-owned commands/adapters:

- Draft Helper create;
- Booking Finder find;
- майбутні triggers додаються як generated typed registration у `mlmail`.

Спільними є selection, lifecycle, CAS load, context boundary, policy та audit; WIT
arguments/results залишаються generated types конкретного trigger.

- [ ] **Step 4: Перевірити task**

Run: `bun run --cwd app test:rust`

Expected: exact release selection детермінована; disabled/uninstalled/wrong-digest target
не викликає guest logic.

---

## Task 5: General-purpose Vue Plugin Manager

**Файли:**

- Modify: `app/src/components/PluginManagerPanel.vue`
- Create: `app/src/components/PluginManagerPanel.vitest.js`
- Modify: Tauri DTOs у `app/src-tauri/src/plugins.rs`
- Generated docs: `app/src/components/docs/PluginManagerPanel.md`

- [ ] **Step 1: Додати component tests до UI changes**

Mock Tauri `invoke` і native file picker. Покрити:

- preflight preview до installation;
- consent rows і confirm/cancel;
- Draft Helper та Booking Finder actions;
- package/version/digest і enabled/lifecycle state;
- exact release передається в action command;
- enable/disable/uninstall оновлюють лише вибраний plugin;
- error не губиться після reload.

- [ ] **Step 2: Прибрати Draft Helper-only повідомлення**

UI пояснює, що приймаються typed `.n-plugin` Components із contracts, підтриманими
поточною версією `mlmail`. Unknown contract показує compatibility error без activation.

- [ ] **Step 3: Рендерити product actions із trigger projection**

Vue не інтерпретує WIT і не запускає Wasm у browser. Backend повертає product-owned
action descriptors, а UI мапить тільки відомі action kinds на typed Tauri commands.

- [ ] **Step 4: Додати explicit release selection**

Кнопка дії кожного list item передає exact release цього item. Не використовувати
глобальний `createDraft()` без target.

- [ ] **Step 5: Перевірити task**

Run: `bun run --cwd app test`

Expected: Vue tests доводять preflight/consent та детермінований вибір plugin release.

---

## Task 6: Required dependency graph installation

**Файли:**

- Modify: `app/src-tauri/src/plugin_install.rs`
- Modify: `app/src-tauri/src/plugins.rs`
- Modify: `app/src-tauri/src/plugin_grants.rs`
- Modify: `app/src-tauri/src/plugin_context.rs`
- Test: graph fixtures з різними publishers у Rust integration tests
- Generated docs: відповідні file docs

- [ ] **Step 1: Замінити empty graph construction**

Не створювати вручну `ResolvedPluginGraph` з одним root і порожніми edges. Використати
`n-plugin-oci` graph resolver, exact `.n-plugin.lock`, local cache та direct OCI backend.

- [ ] **Step 2: Додати online/offline modes**

- online preflight може завантажити відсутні exact dependencies;
- offline activation читає лише lock + verified cache/CAS;
- missing або digest-mismatched dependency зупиняє activation;
- OCI locator лишається provenance, а identity визначається package/version/digest.

- [ ] **Step 3: Застосувати edge-scoped grants**

Dependency без host imports не показує додаткового consent. Dependency з host capability
отримує власний explicit edge grant у тому самому installation preview.

- [ ] **Step 4: Перевірити graph lifecycle**

Disable/uninstall root не видаляє dependency, яку ще досягає інший active root. GC для
source Components використовує mark-and-sweep із quarantine/grace period; LRU стосується
лише regenerable compiled cache.

- [ ] **Step 5: Перевірити task**

Run: `bun run --cwd app test:rust`

Expected: cross-publisher dependency graph активується online, повторно стартує offline,
cycle/digest mismatch/unknown import не змінюють попереднє generation.

---

## Task 7: Authoring і product E2E

**Файли:**

- Modify: `app/scripts/test-booking-finder-component.sh`
- Modify: `app/scripts/test-draft-helper-component.sh`
- Create: `app/scripts/test-installed-plugin-matrix.sh`
- Modify: `app/package.json`
- Modify: релевантні `docs/` у `mlmail`

- [ ] **Step 1: Зібрати незалежні plugin fixtures**

Перевірити щонайменше:

- два publishers із Draft Helper implementation;
- Booking Finder із `mail:search`;
- root Component із required dependency іншого publisher;
- incompatible Component із невідомим trigger/import.

- [ ] **Step 2: Додати installed matrix test**

Matrix має виконати build/package/inspect, preflight, consent, activation, exact typed
invocation, disable/enable, restart offline, uninstall і negative compatibility cases.

- [ ] **Step 3: Провести application E2E**

У release-candidate `mlmail`:

1. встановити два Draft Helper Components;
2. викликати кожен окремо й звірити exact release у result/audit;
3. встановити Booking Finder, отримати consent preview і видати `mail:search`;
4. перевірити online Gmail call;
5. перезапустити без registry/network і перевірити offline activation із CAS;
6. довести, що додавання цих Components не вимагало нового build application.

- [ ] **Step 4: Оновити документацію**

Документувати author flow через `n-plugin new/build/inspect/publish/fetch/lock`, список
підтриманих mlmail contracts, consent semantics, multi-plugin selection, dependency graph,
offline recovery та troubleshooting. Не дублювати platform guide повністю: посилатися на
canonical `n-plugin/docs/plugin-system.md`.

---

## Task 8: Release gates і cutover acceptance

**Файли:** усі змінені файли попередніх tasks.

- [ ] **Step 1: File documentation**

Run: `npx @7n/rules lint doc-files`

Expected: усі змінені `.rs`, `.js` і `.vue` мають актуальні CRC docs; test-файли
враховані як usage evidence.

- [ ] **Step 2: Повний test matrix**

```bash
bun run --cwd app test
bun run --cwd app test:rust
bun run --cwd app test:draft-helper-component
bun run --cwd app test:booking-finder-component
bun run --cwd app test:installed-plugin-matrix
```

- [ ] **Step 3: Delta lint**

Run: `npx @7n/rules lint`

Expected: exit `0`; unrelated pre-existing violations не маскуються новими exclusions.

- [ ] **Step 4: Change-file gate**

Кожен implementation PR створює change-файл через `npx @7n/n ch`; не редагувати
`CHANGELOG.md` або versions вручну.

Run: `npx @7n/rules lint changelog`

Expected: exit `0`.

- [ ] **Step 5: Фінальна acceptance перевірка**

Результат вважається завершеним лише якщо:

- `mlmail` installer не містить Draft Helper-specific validation path;
- два плагіни одного trigger викликаються за exact release;
- Booking Finder встановлюється тим самим UI flow, що Draft Helper;
- supported-contract plugin встановлюється без recompilation application;
- new domain contract потребує змін лише в `mlmail`, не в `n-plugin`;
- dependencies, consent, offline activation і lifecycle пройшли E2E;
- legacy loader, core-Wasm і JSON/string ABI не повернулися.
