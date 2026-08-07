# Domain-agnostic plugin runtime через Dynamic WIT registry

**Дата:** 2026-08-07  
**Статус:** погоджено — готово до реалізації  
**Зв'язані документи:** `docs/specs/2026-08-02-wasm-a2ui-plugin-platform.md`

## 1. Проблема / Мета

Поточний plugin runtime містить mail-specific WIT contract і runtime bridge у
`nitra/tauri-components`. Через це додавання `mail:search` або domain API нового
Tauri-застосунку вимагає зміни shared platform repository. Це змішує generic
platform concerns із продуктово-специфічною логікою та не масштабується на
`mlmail`, `nitra/task` і майбутні host-застосунки.

Потрібна plugin platform, де новий Tauri-застосунок підключає власний versioned
WIT domain API без зміни `nitra/tauri-components`. Плагіни мають зберігати
compile-time typed WIT imports, а platform — єдиний security model: signed
packages, grants, isolation limits, disable/uninstall, audit і A2UI.

Перший product domain — `nitra:mail`; перша нова операція — `mail:search` із
довільним Gmail query, account-scoped consent і metadata-only результатами.

## 2. Ухвалені рішення

| # | Питання | Рішення |
|---|---|---|
| А | Розміщення domain API | Domain WIT contracts належать product repository або окремому domain crate, яким володіє product team. Вони не живуть у platform core за замовчуванням. |
| Б | Підключення domain API | `tauri-components` надає Dynamic WIT registry: host реєструє bindings та handlers власних WIT imports у generic runtime під час старту. |
| В | Межі platform core | Core володіє package lifecycle, signing/trust, Wasm/Wasmtime limits, generic invocation lifecycle, grants storage, audit primitives, A2UI та registry contract. Core не знає `mail:*`, `task:*` або product secrets. |
| Г | Type safety | Plugin imports domain interfaces напряму через WIT. Domain crate генерує typed guest і host bindings; JSON-RPC transport як заміна WIT не використовується. |
| Ґ | Permissions | Domain adapter декларує capability names, allowed scope kinds, consent descriptions та audit policy. Platform зберігає grants і викликає domain policy перед handler-ом. |
| Д | Новий застосунок | Додавання нового host app або domain API не потребує зміни `tauri-components`; app постачає domain crate та реєструє його adapter. |
| Е | Multi-domain plugins | Package може вимагати кілька domain packages. Host запускає його лише коли всі required WIT packages зареєстровані й сумісні; product policy може додатково заборонити поєднання domains. |
| Є | Demo | У `tauri-components` зберігається окремий neutral `Platform Info` demo plugin, який використовує тільки platform-level API та не залежить від `mail` або `task`. |
| Ж | LLM integration | Разом із demo постачається інструкція для LLM/розробника: підключення platform core, registration domain adapter, schemas/capabilities, package verification і test flow. |

## 3. Архітектура

```text
Wasm plugin
  │ typed import: nitra:mail/search@0.1
  ▼
Dynamic WIT registry у tauri-components
  │ resolves domain package + compatibility + invocation lifecycle
  ▼
mlmail MailDomain adapter
  │ validates domain request + checks mail grant + writes audit metadata
  ▼
Gmail API через host-owned OAuth token
```

Для `nitra:task` ланцюжок такий самий, але registry резолвить `nitra:task`, а
`task` host adapter працює зі сховищем задач. Платформа не отримує залежності на
Gmail, task storage або інші product services.

### 3.1. Dynamic WIT registry

Platform core надає public extension contract для domain crates. Його мінімальна
семантика:

1. Domain adapter має stable package identity, наприклад `nitra:mail@0.1.0`.
2. Adapter реєструє у Wasmtime `ComponentLinker` generated WIT host bindings.
3. Adapter надає compatibility metadata для перевірки plugin manifest до запуску.
4. Adapter описує operation-to-capability policy і human-readable consent text.
5. Core викликає adapter у межах стандартних Wasm memory/fuel/timeout/concurrency
   limits і не дозволяє обходити plugin lifecycle або grant store.

Registry відхиляє duplicate package identity та несумісні WIT versions
детермінованою помилкою. Відсутній required domain package також є install/load
error, а не fallback до неtyped виклику.

### 3.2. WIT version compatibility

Plugin manifest декларує потрібні domain packages і supported version range.
Host порівнює цей range з metadata зареєстрованих adapters до першого invoke.

- Backward-compatible additions мають зберігати попередній WIT contract.
- Breaking зміни отримують нову incompatible domain version.
- Host повертає `unsupported-domain` із package name та required/supported version,
  але не розкриває внутрішніх linker details.

### 3.3. Domain isolation

Domain adapter не отримує доступу до runtime state іншого plugin-а. Він знає лише
контекст поточного invoke: plugin identity/version, user identity, grants і
correlation ID. Токени, credential bytes, raw filesystem access та прямий network
access ніколи не передаються у guest Wasm.

## 4. Permissions, consent та audit

### 4.1. Відповідальність domain adapter

Кожен adapter визначає:

- підтримувані capability names;
- допустимі `resource_kind` і правила `resource_id`;
- людський текст для consent UI;
- mutating/read-only classification;
- audit metadata, retention та redaction rules;
- rate, result-size та input-size limits.

Core застосовує ці правила через спільний grant store. Відсутній або невідповідний
grant завершує invoke як `denied` до product service call.

### 4.2. `mail:search`

`mlmail` додає capability:

```text
name: mail:search
scope: account
```

Grant надає плагіну право виконувати довільні Gmail search queries в обраному
account. Query не входить до scope і не обмежується allowlist-ом. Обмеження
безпеки й ресурсу застосовуються незалежно від query: максимальна довжина query,
maximum page/result count і per-plugin rate limit.

Операція повертає тільки metadata:

```text
id, from, subject, date
```

Вона не повертає body, attachment bytes, Gmail OAuth token, raw HTTP response
або credential material. Audit запис містить plugin, capability, account scope,
result status, correlation ID та кількість результатів; повний query і metadata
листів не зберігаються в audit log.

## 5. Product domains

### 5.1. `mlmail`: `nitra:mail`

`mlmail` створює product-owned `nitra:mail` domain crate з WIT definitions,
generated bindings, manifest capability policy та Gmail adapter.

Початкові операції:

- `mail:metadata.read`;
- `mail:draft.create`;
- `mail:search`.

`mail:search` виконується host-ом через існуючий Gmail search flow, але plugin
викликає тільки typed WIT import. Adapter передає Gmail OAuth token лише в
host-owned Gmail client.

### 5.2. `nitra:task`

`nitra:task` є independent domain crate. Він сам визначає WIT records, operations
та capabilities, наприклад `task:search`, `task:read`, `task:create` і
`task:update`. Зміни `nitra:task` не змінюють `nitra:mail` або platform core.

## 6. Neutral demo components

`tauri-components` містить `demo-components/platform-info` як signed sample
package та source для його збирання. Demo використовує тільки platform-level WIT
API `platform:get-info`.

`platform:get-info` повертає виключно безпечну application metadata:

- application name;
- application version;
- platform API version;
- registered domain package identities та versions.

Відповідь не містить account identifiers, install paths, secrets, token values,
filesystem data або product domain records. Demo потрібен для acceptance testing
platform integration в будь-якому Tauri host і не замінює product-specific samples.

## 7. Інструкція для LLM і розробника

У `tauri-components` додається коротка integration guide з детермінованим
workflow:

1. Підключити platform core до Tauri builder.
2. Реалізувати product-owned WIT domain crate.
3. Описати capability policy, consent text, audit redaction і compatibility metadata.
4. Зареєструвати adapter у Dynamic WIT registry до завантаження plugins.
5. Перевірити `Platform Info` demo plugin.
6. Додати product contract та integration tests.

Guide має явно забороняти передачу credentials у guest, bypass grant checks,
невалідувані dynamic imports і перевірку UI лише скриншотом. Для Tauri verification
вона використовує DOM accessibility snapshot, direct invoke та console-log checks.

## 8. Міграція

1. Виділити mail-specific WIT, `MailHost` bridge і sample plugin з platform core
   у `mlmail` domain crate.
2. У platform core ввести Dynamic WIT registry і neutral platform WIT API.
3. Додати compatibility adapter для поточного `Draft Helper` або одночасно
   перепакувати його під product-owned `nitra:mail` contract.
4. Перевести `mlmail` на registry registration та перевірити весь existing draft
   flow без регресії.
5. Додати `mail:search` і `Booking Finder` sample plugin як першу доказову
   product feature.
6. Перевести `nitra:task` окремо; його міграція не блокує `mlmail`.
7. Після міграції видалити deprecated mail-specific exports з platform core.

Міграція проходить за feature flag, доки `mlmail` не підтвердить registry path у
debug та integration tests. Release build не містить development bridge або
debug-only bypass capability checks.

## 9. План реалізації

1. Спроєктувати і протестувати public Rust extension contract для registry у
   `tauri-components`, включно з duplicate/unsupported-domain errors.
2. Винести generic runtime, grants, audit та A2UI в domain-agnostic layers;
   прибрати mail types з public core API.
3. Додати platform-level WIT package та `Platform Info` demo-components package.
4. Написати integration guide і test fixture Tauri host, який реєструє test domain.
5. Створити в `mlmail` crate `nitra:mail`: WIT, generated bindings, capability
   policy, Gmail adapter і contract tests.
6. Мігрувати `Draft Helper` та додати `mail:search` / `Booking Finder`.
7. Перевірити consent, denied grant, disabled plugin, incompatible domain,
   result limits, audit redaction і uninstall purge через unit, integration та
   Tauri MCP tests.
8. Додати `nitra:task` як незалежну product integration після стабілізації
   registry contract.

## 10. Критерії приймання

- Новий Tauri host реєструє test domain без зміни `tauri-components` source.
- Plugin із typed import у зареєстрований domain успішно виконує handler.
- Plugin із відсутнім або несумісним domain package не запускається з
  детермінованою помилкою.
- Відсутній grant блокує domain operation до виклику product service.
- Disabled plugin не може викликати жоден domain adapter.
- `Platform Info` demo працює у `mlmail`, `task` і мінімальному fixture host без
  product-specific dependencies.
- `mail:search` із account grant повертає лише metadata; без grant повертає
  `denied`; audit не містить query або content листа.
- Жоден тест або runtime path не передає OAuth token, raw HTTP або filesystem
  capability у guest Wasm.

## Відкриті питання

- Остаточна Rust форма type-erased registration API для generated WIT bindings у
  Wasmtime `ComponentLinker`.
- Exact policy для cross-domain plugins: deny-by-default або product-configurable
  allowlist у першому релізі.
- Версійна схема domain ranges у plugin manifest і правила compatibility між WIT
  minor versions.
- Чи потрібен streaming/pagination protocol для `mail:search` у першій версії,
  чи достатньо bounded list result.

## 11. Plugin dependencies і plugin-to-plugin calls

### 11.1. Мета та модель

Встановлений Wasm plugin може викликати інший встановлений Wasm plugin лише як
явно задекларовану dependency. Це не є довільним discovery усіх plugins і не є
звичайним npm import: platform завжди лишається посередником, який перевіряє
identity, compatibility, lifecycle і permissions кожного учасника.

Dependency може належати іншому publisher. Кожен package зберігає власні
signature, publisher key, fingerprint, trust state, manifest, capabilities і
grants. Root plugin не передає dependency свої permissions, і dependency не
передає permissions root plugin-у.

```text
Root plugin A
  │ typed WIT import до dependency B
  ▼
Dynamic WIT registry
  │ verifies declared edge, version and lifecycle
  ▼
Dependency plugin B
  │ виконується як B, з grants B
  ▼
Product domain adapter
```

### 11.2. Manifest contract

Plugin manifest отримує декларативний dependency section. Кожен dependency
описує:

- immutable plugin identifier;
- supported version range;
- required або optional status;
- потрібний exported WIT package/interface та його compatible version range;
- allowed package source constraints, якщо product policy їх підтримує.

Plugin може імпортувати лише WIT exports dependencies, які перелічені у власному
manifest. Runtime не надає API для переліку або виклику довільних installed
plugins. Це запобігає confused-deputy flows і неявному формуванню нових trust
relationships після інсталяції.

### 11.3. Resolution: registry-first з lockfile

Основна модель — registry-first resolver. Root manifest містить version ranges,
а installer резолвить повний dependency graph із product-approved registry або
іншого явно дозволеного source. Bundled signed packages у root `.n-plugin`
допускаються як optional offline/cache source, але не є єдиним способом доставки.

Після resolution platform створює lockfile з точними значеннями для кожного
node:

- plugin ID і resolved version;
- package checksum;
- publisher key fingerprint;
- source identity;
- WIT package/interface version;
- dependency edges.

Повторна інсталяція або update використовує lockfile, доки користувач явно не
погоджується на graph update. Resolver не підміняє package іншим publisher key
або іншим source лише через однаковий plugin ID.

Перед записом у active registry resolver:

1. будує повний dependency graph;
2. відхиляє cycle, duplicate incompatible version і недоступний required node;
3. застосовує graph depth, node-count і package-size limits;
4. завантажує або бере локальний package лише з дозволеного source;
5. перевіряє signature, checksum і publisher trust для кожного node;
6. перевіряє WIT import/export compatibility;
7. готує один consent/install preview для всього graph.

Installation відбувається через staging transaction. Якщо install, signature,
manifest, consent або registry activation будь-якого node не пройшли, platform
робить rollback усього tree й не лишає partial active graph.

### 11.4. Multi-publisher trust і consent

Consent UI показує розгортуваний dependency tree. Для кожного plugin користувач
бачить name, exact version, publisher, public-key fingerprint, source, required
WIT interface, requested capabilities і статус required/optional.

New publisher key або key rotation потребує окремого явного acceptance у межах
одного tree consent flow. Unsigned packages заборонені у release builds, зокрема
для transitive dependencies.

Grants обробляються per-plugin:

- code-only dependency може не мати capabilities;
- dependency з capability отримує власний consent і власний grant record;
- caller не може використати `mail:search` лише тому, що його dependency має цей
  grant;
- dependency не може використати grant caller-а.

Product policy може відхилити dependency tree з поєднанням високоризикових
capabilities або sources, але така policy має бути видимою користувачу як причина
відмови, а не тихим filter-ом.

### 11.5. Runtime invocation

Plugin-to-plugin calls використовують typed WIT imports/exports через Dynamic
WIT registry. Перед invocation platform перевіряє:

1. caller має declared dependency edge до callee;
2. callee installed, enabled і compatible з lockfile;
3. caller і callee WIT interfaces сумісні;
4. nested invocation depth не перевищує configured limit;
5. call chain не містить cycle або re-entrant invoke того самого plugin.

Callee запускається в окремому Wasm invocation context під власними plugin ID,
version і grants. Він не має доступу до memory caller-а, до його приватних
settings або до raw input поза typed WIT request. Memory, fuel, timeout,
concurrency й output-size limits застосовуються до кожного invoke.

Усі помилки мають stable safe codes: `dependency-missing`,
`dependency-disabled`, `dependency-incompatible`, `dependency-cycle`, `denied`,
`rate-limited` і `unavailable`. Internal linker, OAuth, filesystem або network
details не потрапляють у Wasm guest error.

### 11.6. Audit

Platform створює correlation chain для кожного nested invoke, наприклад:

```text
root-plugin-A → dependency-B → nitra:mail/search
```

Audit фіксує caller, callee, operation, capability, scope, result status,
correlation ID і timing metadata. Він не зберігає bodies, OAuth tokens, raw
request payloads або приватні дані domain operation. Mutating domain actions
далі підлягають product-specific audit policy.

### 11.7. Lifecycle і Plugin Manager

Plugin Manager показує для кожного node:

- `Depends on` — direct dependencies;
- `Used by` — direct dependents;
- health: installed, enabled, compatible, granted або blocked;
- resolved version, publisher fingerprint і source;
- optional/required статус.

Disable dependency блокує всі dependent calls із `dependency-disabled`. Required
dependency робить root plugin non-invocable; optional dependency дозволяє root
plugin-у перейти у явно описаний degraded mode.

Uninstall dependency, що має active dependents, не виконується мовчки. Manager
показує dependents і пропонує видалити/оновити їх разом або скасувати дію. Shared
dependency інсталюється один раз: registry зберігає dependency edges, а не
небезпечний неявний reference count.

Update root plugin показує diff повного graph: added/removed/changed packages,
versions, publisher keys, sources, WIT interfaces і capability requests. Після
підтвердження update активується атомарно; failure повертає попередній lockfile
та active graph.

### 11.8. План реалізації dependencies

1. Розширити package manifest dependency declarations і WIT export metadata.
2. Реалізувати deterministic graph resolver, source policy, lockfile і cycle /
   compatibility validation.
3. Додати staged atomic install, rollback та registry storage dependency edges.
4. Розширити trust/consent preview для multi-publisher tree.
5. Додати plugin-to-plugin invocation route до Dynamic WIT registry з isolated
   callee identity, bounded nesting та correlation chain.
6. Оновити Plugin Manager: dependency tree, health, `Used by`, uninstall/update
   protection.
7. Додати fixtures: same-publisher, multi-publisher, optional dependency,
   offline bundled package та separately downloaded package.
8. Додати LLM integration guide: dependency declaration, lockfile review,
   grants isolation і заборона dynamic plugin discovery.

### 11.9. Критерії приймання dependencies

- Root plugin з dependency іншого publisher показує повний tree до install і
  вимагає acceptance кожного нового publisher key.
- Resolver окремо завантажує signed dependency з approved registry, фіксує exact
  graph у lockfile та повторно використовує його без silent update.
- Invalid signature, cycle, incompatible WIT interface або missing required node
  не залишає partial installation.
- Plugin не може викликати installed plugin, якого немає у його manifest
  dependencies.
- Dependency call використовує callee grants; grants caller-а не успадковуються.
- Audit відображає complete caller-to-callee chain без body, token або raw query.
- Disable/uninstall/update dependency коректно показує і захищає dependents.
- Optional dependency дає documented degraded behavior; required dependency
  блокує root invocation.
