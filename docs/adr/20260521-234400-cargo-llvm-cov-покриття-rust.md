# Встановлення cargo-llvm-cov для покриття Rust-коду

**Status:** Accepted
**Date:** 2026-05-21

## Context and Problem Statement

`cargo test` виконує тести, але не рахує покриття. У `src-tauri/src` міститься 71 тест у 11 файлах, тому потрібен інструмент для вимірювання покриття Rust-коду без зовнішнього CI-сервісу.

## Considered Options

- `cargo-llvm-cov` (LLVM source-based coverage)
- `cargo-tarpaulin`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "`cargo-llvm-cov`", because це стандартний і точний інструмент на основі LLVM source-based coverage; користувач обрав варіант «встановити й прогнати».

### Consequences

- Good, because звіт покриття по рядках для всіх `src-tauri/src/**/*.rs` доступний через `bun run test:rust:coverage`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- Команди встановлення: `rustup component add llvm-tools-preview`, `cargo install cargo-llvm-cov`
- Версія `cargo-llvm-cov`: `0.8.7`
- Новий скрипт у `app/package.json`: `"test:rust:coverage": "cargo llvm-cov --manifest-path src-tauri/Cargo.toml"`
- Директорія `app/src-tauri/target/llvm-cov-target` покривається наявним `.gitignore`-патерном `target/`
