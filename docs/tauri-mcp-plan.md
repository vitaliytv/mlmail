# План впровадження MCP-доступу до dev console у Tauri v2 застосунку

**Проєкт:** `/Users/vitalii/www/vitaliytv/mlmail`
**Мета:** дати Claude (через MCP) доступ до dev console (логи, DOM, IPC) Tauri-застосунку для дебагу.

---

## Контекст і прийняте рішення

Розглянуто кілька готових рішень і власну розробку. Підсумок рішення:

**Починаємо з `hypothesi/mcp-server-tauri`** (найзріліший з готових варіантів), а власний rmcp-based (Rust-native) сервер розглядаємо пізніше, коли назбирається досвід реального використання.

### Чому hypothesi, а не альтернативи

| Критерій | hypothesi/mcp-server-tauri | P3GLEG/tauri-plugin-mcp | moinsen-dev (форк P3GLEG) |
|---|---|---|---|
| ⭐ Stars | 230 | 100 | менш видимий |
| Комітів | 190 | 33 | — |
| Релізів | 28 (кейденс ~раз/тиждень) | 1 | — |
| Опубл. в npm | ✅ `@hypothesi/tauri-mcp-server` | ❌ (треба збирати локально) | ❌ (треба збирати локально) |
| Console/error tracking | ✅ вбудовано з коробки | ❌ нема в оригіналі | ✅ додано у форку |
| Транспорт | WebSocket, порт 9223 | Unix socket / TCP | Unix socket / TCP |
| Мультизастосунок (кілька Tauri-апп одночасно) | ✅ підтримується явно | — | — |
| localStorage CRUD-тули | не окремо | ✅ | ✅ |
| Платформи | desktop + Android logcat + iOS simulator | desktop | desktop |

**Аргумент на користь hypothesi:** зрілий release-цикл, готовий npm-пакет (не треба тримати build-крок), явна підтримка кількох застосунків одним MCP-сервером, вбудований console/error tracking.

**Аргумент, який міг би схилити до moinsen-dev/P3GLEG:** легший транспорт (без `window.__TAURI__` глобально), явний localStorage tooling, Context7-інтеграція. Але це community-форк без власного release-ритму — менш надійний для довгострокової залежності.

### Чому НЕ власний rmcp (Rust-native) сервер — поки що

Розглядали заміну Node-прошарку на власний MCP-сервер на офіційному Rust SDK (`rmcp`). Три причини відкласти:

1. **Транспорт не пасує до GUI-процесу.** MCP-клієнти (зокрема **Zed — тільки stdio**, без нативного HTTP/SSE) очікують або stdio (спавнять процес самі), або HTTP/SSE з окремим discovery. Вбудовування HTTP-сервера прямо в GUI-процес Tauri-застосунку для Zed однаково вимагає `mcp-remote` proxy (знову Node) — реальної переваги немає.
2. **Release-цикл MCP-шару не варто змішувати з релізним циклом застосунку.** `rmcp` після виходу 1.0 (03.2026) досі має кейденс ~раз на 1-2 тижні — вбудовування напряму means постійний `cargo update` + перевірка сумісності всередині кодбейзу продукту.
3. **Документаційне покриття rmcp — лише ~40%** (747/1879 елементів задокументовано на docs.rs) — вища вартість розробки з нуля порівняно з готовими інструментами.

Додатковий факт: жоден з існючих community-плагінів (hypothesi, P3GLEG, moinsen-dev) не використовує rmcp навіть зараз — усі тримають Rust-сторону як простий сокет-сервер, а MCP-протокольну логіку — окремо в Node.

**Клієнтська підтримка транспортів (для довідки, якщо повертатись до цього питання):**

| Клієнт | stdio | HTTP/SSE |
|---|---|---|
| Zed | ✅ нативно | ❌ лише через `mcp-remote` proxy |
| Pi CLI (з `pi-mcp-extension`) | ✅ | ✅ нативно, `streamable-http` і legacy `sse` |

Pi.dev (Mario Zechner) в ядрі MCP не підтримує принципово (context bloat філософія), але офіційне розширення `pi-mcp-extension` додає повноцінний MCP-клієнт з усіма трьома транспортами.

---

## Кроки впровадження

### 1. Передумови

```bash
node --version   # 20+
cargo --version  # Rust toolchain
npm install -g @tauri-apps/cli@next
```

### 2. Rust-плагін у `src-tauri`

```bash
cd src-tauri
cargo add tauri-plugin-mcp-bridge
```

У `src-tauri/src/main.rs` (або `lib.rs`, якщо є мобільні таргети):

```rust
fn main() {
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Критично:** плагін лише під `#[cfg(debug_assertions)]` — перевірити, що build-пайплайн CI/CD не знімає цей guard для release-збірок.

### 3. `tauri.conf.json`

`app/src-tauri/tauri.conf.json` — єдиний конфіг проєкту, використовується і для `tauri dev`, і для release-збірки (CI бере з нього версію тощо) — його не чіпаємо.

Замість цього `withGlobalTauri` винесено в окремий dev-only overlay `app/src-tauri/tauri.conf.dev.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "app": {
    "withGlobalTauri": true
  }
}
```

і підключається лише в dev-скрипті (`package.json#scripts.start`) через `--config`:

```json
"start": "bun --cwd=app run tauri dev --config src-tauri/tauri.conf.dev.json"
```

`tauri build` (і CI release pipeline) цей файл не бачить — `window.__TAURI__` глобально відкривається лише в `tauri dev`, ніколи в release-збірці.

### 4. Capabilities

Додати дозволи плагіна в `src-tauri/capabilities/default.json`. Плагін розрахований як єдиний блок дозволів — часткові дозволи (тільки читання логів без DOM-доступу тощо) не підтримуються в поточній версії.

### 5. Конфіг MCP-клієнта (Zed)

```json
{
  "mcpServers": {
    "tauri": {
      "command": "npx",
      "args": ["-y", "@hypothesi/tauri-mcp-server"]
    }
  }
}
```

### 6. Перевірка

- Рестарт Zed.
- Slash-команда `/setup` — гайдед-налаштування.
- `ping`-тул — перевірка з'єднання.
- Пілотний виклик: `read_console_logs` (чи еквівалент з набору 21 інструменту hypothesi) — перевірити весь ланцюжок plugin → WebSocket → MCP-сервер → Zed.

### 7. Порти і кілька застосунків

WebSocket за замовчуванням — порт **9223**. Якщо колись паралельно дебажиться кілька Tauri-застосунків (наприклад mlmail + інший проєкт) — MCP-сервер hypothesi підтримує керування кількома з'єднаннями одночасно через `driver_session` з параметром host/port; окремо реєструвати кілька MCP-серверів у конфізі не потрібно.

---

## Список 21 інструменту hypothesi (4 категорії)

- **Setup** (1) — `/setup` slash-команда
- **UI Automation & WebView** (14) — скріншоти, DOM-снапшот, виконання JS, стрім console-логів (desktop + Android logcat + iOS simulator), керування вікнами
- **IPC Tools** (5) — моніторинг Tauri commands/events між фронтендом і Rust-бекендом у реальному часі
- **Mobile** (1) — список підключених пристроїв/емуляторів

---

## Наступні кроки (відкрите питання)

Обрати, на чому обкатати пілот `read_console_logs` в `mlmail`:
- напряму на робочому коді застосунку, чи
- спершу мінімальний тестовий екран/компонент усередині того ж проєкту, щоб перевірити ланцюжок без ризику для робочого коду.

## Коли повертатись до rmcp-варіанту

Після зрілого використання hypothesi (реальний досвід — які тули насправді потрібні, наскільки заважає Node-залежність, чи є проблеми з release-кейденсом hypothesi) — переоцінити власний rmcp-based сайдкар, орієнтуючись на:
- чи справді потрібна незалежність від Node/TS у критичному шляху;
- чи з'явилась нативна HTTP-підтримка в Zed (наразі відкритий feature request);
- чи виправдовує обсяг використання вартість підтримки власного MCP-сервера.
