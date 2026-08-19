---
type: Rust Module
title: plugin_booking_finder.rs
resource: app/src-tauri/src/gmail/plugin_booking_finder.rs
docgen:
  crc: adaef35b
  model: manual
---

## Огляд

Generated typed adapter між Booking Finder Component world і native Gmail search host.

## Поведінка

Adapter компілює перевірені Component bytes і створює instance у product runtime, підключає typed Gmail/WASI imports та повертає generated `BookingResults`. Exact release, lifecycle і generic `mail:search` grant перевіряє product dispatcher до передачі OAuth token.

## Публічний API

- `invoke_booking_finder` — викликає typed `find` export і повертає query та Gmail message references.

## Гарантії поведінки

- Dynamic JSON broker не використовується.
- OAuth token зберігається лише в host state на час invocation.
- Grant policy не виводиться з Component bytes усередині adapter.
