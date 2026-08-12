---
type: ADR
title: "Окремий coordinated monorepo n-plugin"
description: "Винесення всієї Tauri/Vue plugin system із tauri-components в repository nitra/n-plugin з єдиним release profile."
---

**Status:** Accepted
**Date:** 2026-08-08

## Context and Problem Statement

Специфікація plugin system охоплює Component runtime, WIT registry, package
resolution, immutable dependency graphs, CAS/SQLite, grants, audit, application
updates, Vue UI, author SDK, demo, documentation і `n-plugin-cli`. Це окрема
предметна область і release lifecycle, які більше не відповідають межі
`nitra/tauri-components` як бібліотеки generic Vue/Tauri components.

Потрібно визначити repository boundary, внутрішню структуру й release model так,
щоб plugin platform не створила circular dependency з `tauri-components`, але
Rust, Vue, WIT і CLI artifacts лишалися сумісними.

Обмеження: усі supported applications використовують Tauri та Vue; підтримка
інших frameworks або non-Tauri hosts не є ціллю. Legacy compatibility facade не
потрібний.

## Considered Options

- Coordinated platform monorepo: modular Rust/npm/WIT/CLI workspace з однією
  platform version та одним tested compatibility set.
- Independently versioned workspace: один repository, але окремий SemVer і
  compatibility matrix для кожного crate/package.
- Integrated platform package: один великий Rust crate, один Vue package і CLI з
  мінімальною внутрішньою модульністю.

## Decision Outcome

Chosen option: "Coordinated platform monorepo", because plugin runtime, WIT
contracts, Vue UI та author toolchain повинні змінюватися атомарно, а modular
workspace зберігає внутрішні boundaries без ранньої compatibility matrix.

Новий repository має назву `nitra/n-plugin` і містить всю
plugin-specific реалізацію. `nitra/tauri-components` лишається generic UI/Tauri
library. Plugin platform може односторонньо залежати від її public primitives і
design tokens; зворотна залежність заборонена.

M0 Component Model spike одразу створюється в новому repository. Тимчасова
реалізація в `tauri-components`, compatibility facade або re-exports не потрібні.

### Consequences

- Good, because repository ownership відповідає окремій предметній області.
- Good, because одна platform version задає exact compatible Rust, npm, WIT і
  CLI artifacts.
- Good, because product domains далі додаються без змін platform repositories.
- Good, because plugin-specific dependency weight і release cadence більше не
  навантажують generic `tauri-components`.
- Bad, because product applications мають координувати dependencies на два
  shared repositories.
- Bad, because навіть локальна Vue-зміна може створити coordinated platform
  release.
- Bad, because extraction потребує окремого CI, publishing і ownership setup.

Success signal: `mlmail` і neutral fixture host використовують один released
platform profile; `mlmail` реєструє `nitra:gmail` без зміни platform repositories;
CI доводить відсутність dependency `tauri-components → n-plugin`.

## More Information

Усі ідеї brainstorming-сесії:

1. Окремий repository `nitra/n-plugin`.
2. Власний release lifecycle.
3. Власний compatibility profile.
4. Повністю прибрати plugin-specific code із `tauri-components`.
5. Дозволити односторонню залежність від generic Vue primitives.
6. Заборонити зворотну залежність із `tauri-components`.
7. Product applications залежать від обох repositories напряму.
8. Rust workspace із кількома internal crates.
9. Окремий Tauri host crate.
10. Окремий Component runtime crate.
11. Окремий activation graph crate.
12. Окремий `WkgResolutionBackend`, який вбудовує
    `wasm-pkg-core`/`wasm-pkg-client` як єдиний package version resolver і не
    запускає зовнішній `wkg` executable.
13. Окремий CAS/storage crate.
14. Окремий grants/policy crate.
15. Окремий structured audit crate.
16. Окремий application compatibility crate.
17. Vue Plugin Manager як npm package.
18. Vue A2UI renderer як npm package.
19. Tauri frontend bindings як npm package.
20. `n-plugin-cli` у Rust workspace.
21. Canonical WIT packages у `wit/`.
22. `.n-plugin.toml` schema у platform repository.
23. Generated Rust host bindings.
24. Generated TypeScript frontend bindings.
25. Одне джерело contracts без ручного дублювання types.
26. Local Component test host.
27. Conformance test suite для product applications.
28. Neutral `Platform Info` demo.
29. Dependency graph fixtures.
30. Multi-publisher fixtures.
31. Async stream fixtures.
32. Application update compatibility fixtures.
33. Reference minimal Tauri/Vue host.
34. Reference plugin author project.
35. Host integration SDK.
36. Product-local domain adapter SDK.
37. Generated `PluginHostInterfaceRegistry`.
38. Generated `ApplicationTriggerSet`.
39. Public API inventory, який перевіряється CI.
40. CI rule, що забороняє circular dependency.
41. Окремі CI lanes для Rust, Vue, WIT, CLI і conformance.
42. Єдиний release manifest для всіх artifacts.
43. Unified platform version для Rust/npm/WIT/CLI.
44. Незалежні package versions із compatibility matrix.
45. Exact toolchain snapshot як release artifact.
46. Versioned documentation у `docs/`.
47. Architecture decision records у platform repository.
48. `CODEOWNERS` для runtime, UI, WIT і CLI.
49. Breaking extraction без compatibility facade.
50. M0 spike реалізувати одразу в новому repository.

Відкладено independently versioned workspace через передчасну compatibility
matrix. Integrated package відкладено, бо воно відтворює надмірно тісну межу,
через яку plugin system виноситься з `tauri-components`.

Відкритих питань щодо repository boundary немає. Exact internal crate/package
layout уточнюється implementation design без зміни цього рішення.
