# Tool Surface для mlmail: LLM-доступний шар бекенду з локальними моделями per-platform

**Status:** Accepted
**Date:** 2026-06-15

## Context and Problem Statement

Бачення mlmail (README) — це AI-воронка над листом: саммері → озвучка → дії (видалити / фільтр / зберегти в `work`|`home` / чернетка відповіді). Для цього шар бекенду має бути викликабельним не лише з UI, а й з LLM. Наразі фронтенд б'є в Rust напряму через сирий `invoke` в `auth-store.js` — кожна дія досяжна лише через UI-взаємодію, дублювалась би при додаванні LLM, і паритет поведінки не гарантований. Споріднений проєкт `nitra/task` уже вирішив це через архітектуру `n-tool-surface` (catalog → dispatch → адаптери), яку вирішено перенести.

## Considered Options

- Перенести `n-tool-surface` з `nitra/task`: спільний каталог тулів зі схемами, `dispatch` з уніфікованим конвертом, маніфест для LLM; UI і LLM — рівноправні адаптери.
- Лишити прямі `invoke` і додати окремий LLM-шлях паралельно до UI.
- LLM-провайдер: хмарний Claude через `tauri-plugin-http` (кросплатформно, але онлайн і потребує ключа).
- LLM-провайдер: локальні моделі per-platform — omlx (macOS) + LiteRT-LM (Android).

## Decision Outcome

Chosen option: "перенести `n-tool-surface` + локальні моделі per-platform", because будь-яку дію фронтенду має бути виконувано без UI через той самий `dispatch`; LLM — ключовий новий споживач, тому термінологія `tool` і маніфест у OpenAI function-calling форматі. LLM працює **локально на обох платформах** (приватність, офлайн): на десктопі — **omlx** (OpenAI-сумісний MLX-сервер на `localhost`, як у `task`), на **Android 16+** — **LiteRT-LM** (наступник задепрекейченого MediaPipe LLM Inference API; Gemma 3n E2B/E4B on-device). Розбіжність реалізації ховається за інжектованим `chat`-адаптером — `dispatch` (тул-сурфейс) однаковий скрізь.

### Consequences

- Good, because наявні Gmail/auth-команди загорнуто в каталог (`is_authenticated`, `current_email`, `inbox_count`, `random_message`, `random_newsletter`, `unsubscribe`); `auth-store.js` переведено з прямих `invoke` на `dispatch`; 85/85 JS-тестів проходять.
- Good, because нові AI-фічі (summarize / draft_reply / delete / create_filter / save_to_memory) додаються як нові тули в один каталог — README-воронка стає agent-loop без дублювання.
- Good, because інжекція `chat` дозволяє один `runAgent` для omlx (HTTP), LiteRT-LM (через нативний міст) і тестів (mock); конверт зберігає backend-`kind` (`ReauthRequired` тощо), тож UX помилок не змінився.
- Bad, because Android потребує нового Tauri Kotlin-плагіна-моста до LiteRT-LM (у `task` цього не було — він був macOS-only).
- Neutral, because агентний tool-calling надійний на десктопі (omlx function-calling), а на Android — під питанням; старт на Android — через task-specific пайплайн (summarize → classify → propose), а вільний агент-loop лишається десктопу.
- Neutral, because `npx @nitra/cursor` підняв `@nitra/cursor` з `^1.27.5` до `^11.1.0` і виконав `bun i` (правило `n-tool-surface.mdc` авто-активується на `@tauri-apps/api`).

## More Information

**Свідома відмінність від канону `n-tool-surface`:** CLI/оркестратор-споживач **виключено** — у mlmail немає окремого бінарника (бекенд суто Tauri-команди), тож тул-сурфейс має два споживачі (UI + LLM), а не три. `bin/`-частина і MCP-stdio з `task` не переносяться.

**Файли:** `app/src/tool/{catalog,dispatch,manifest,transports,index}.js`, `app/src/tool/tool.test.js`; переписано `app/src/services/auth-store.js`. Конверт: `{ ok, output }` / `{ ok:false, error:{ code, message, kind } }`.

**TTS («озвучити саммері»)** — нативний синтез (Android `TextToSpeech`, macOS `AVSpeechSynthesizer`) як окрема Tauri-команда `speak`, не через `chat`.

**Наступні кроки:** `llm.js` (`runAgent` + `createOpenAiChat` для omlx); Android Kotlin-плагін → LiteRT-LM + `chat`-адаптер; нові AI-тули; confirm-гейт для `scope: "mutate"`.
