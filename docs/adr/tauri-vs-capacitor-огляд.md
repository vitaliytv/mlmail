# Tauri vs Capacitor: технічний огляд

**Status:** Accepted
**Date:** 2026-05-14

## Context and Problem Statement

Команда розглядає Tauri як альтернативу Capacitor для десктопних і мобільних застосунків у контексті проєкту MLMaiL. Потрібно пояснити відмінності, переваги та обмеження для технічного колективу.

## Considered Options

- Tauri — системний WebView + Rust-бекенд, Android NDK без JVM
- Capacitor — Cordova-подібна обгортка над WebView, JS-міст, Java/Kotlin плагіни
- Electron — вбудований Chromium, більший бінарник

## Decision Outcome

Chosen option: "Tauri", because він дає менший бінарник (без вбудованого Chromium), прямий доступ до нативних API через Rust FFI і можливість запускати локальні ML-моделі без Bridge-оверхеду.

### Consequences

- Good, because на Android Tauri використовує Android NDK через Rust без JVM/Java у критичному шляху.
- Good, because локальний AI (llama.cpp, ort) запускається у Rust-процесі без IPC bottleneck.
- Good, because розмір бінарника значно менший порівняно з Electron/Capacitor desktop.
- Bad, because Capacitor виграє, коли команда суто JS, потрібен Cordova ecosystem або проєкт суто mobile-first без важких обчислень.
- Neutral, because Tauri не має web-платформи — frontend деплоїться окремо через Vite як PWA.

## More Information

Огляд підготовлений для команди в контексті вибору фреймворку для нових проєктів (desktop+mobile), Android NDK vs JVM та інтеграції локального AI. Деталі щодо відсутності web-таргета та архітектурних наслідків — у ADR `tauri-web-таргет-відсутність.md`.
