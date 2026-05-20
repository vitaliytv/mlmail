# rust-analyzer не знаходить workspace у кореневій директорії

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

IDE відображала повторювані помилки `FetchWorkspaceError: rust-analyzer failed to fetch workspace` та `failed to find any projects in [AbsPathBuf("/Users/vitaliytv/www/vitaliytv/mlmail")]`. Проєкт має монорепо-структуру: Vue-фронтенд у `app/`, Rust/Tauri-бекенд у `app/src-tauri/`. `Cargo.toml` знаходиться в `app/src-tauri/`, а не в корені репозиторію, тому rust-analyzer, запущений з кореня, не може автоматично виявити проєкт. Додатково фіксувалися попередження про повільні цикли обробки подій (121–287 мс на `textDocument/didChange` / `didClose` / `didOpen`).

## Considered Options

- Додати `"rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"]` у `.vscode/settings.json`.
- Відкривати `app/src-tauri/` як окремий workspace у VS Code.
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Налаштувати linkedProjects у .vscode/settings.json", because це мінімальна зміна, що дозволяє зберігати єдиний workspace VS Code у корені репозиторію та одночасно вказати rust-analyzer на правильний `Cargo.toml`.

### Consequences

- Good, because rust-analyzer знаходить `Cargo.toml` і більше не генерує `FetchWorkspaceError`.
- Neutral, because конфіг `.vscode/settings.json` потрібно підтримувати при зміні структури директорій.
- Bad, because transcript не містить підтвердження щодо усунення попереджень про повільні цикли обробки подій (121–287 мс).

## More Information

Зачіпає: `app/src-tauri/Cargo.toml` (цільовий файл для linkedProjects), `.vscode/settings.json` (необхідно додати `"rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"]`). Сесія містить лише пасивне спостереження логу IDE — жодних дій у коді не виконувалося.

---

**Опрацьовано** 2026-05-20. Проекції:
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
