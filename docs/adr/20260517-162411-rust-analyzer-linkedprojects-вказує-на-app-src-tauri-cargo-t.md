---
session: fb4e0ff3-d91d-4e9d-bb39-fd6541f6127a
captured: 2026-05-17T16:24:11+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/fb4e0ff3-d91d-4e9d-bb39-fd6541f6127a.jsonl
---

## ADR rust-analyzer: linkedProjects вказує на `app/src-tauri/Cargo.toml`

**Контекст:** rust-analyzer у VS Code не знаходить workspace у кореневій директорії проєкту (`/Users/vitaliytv/www/vitaliytv/mlmail`), бо `Cargo.toml` розташований не в кореневій, а у вкладеній директорії `app/src-tauri/`. Це спричиняло помилку `FetchWorkspaceError`.

**Рішення/Процедура/Факт:** У файл `.vscode/settings.json` додано налаштування `"rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"]`, яке явно вказує rust-analyzer на розташування `Cargo.toml`. Після змін потрібно перезапустити rust-analyzer через Command Palette («rust-analyzer: Restart Server» або «Reload Window»).

**Обґрунтування:** Коли Rust-проєкт знаходиться не в кореневій директорії репозиторію (монорепо або Tauri-проєкт у підпапці `app/`), rust-analyzer не може автоматично виявити workspace. Явне посилання через `linkedProjects` вирішує проблему без зміни структури репозиторію.

**Розглянуті альтернативи:** Не обговорювалися; рішення взято з наявного ADR у `docs/adr/20260517-105342-rust-analyzer-не-знаходить-workspace-у-кореневій-директорії.md`.

**Зачіпає:** `.vscode/settings.json`, `app/src-tauri/Cargo.toml`, налаштування rust-analyzer у VS Code.
