# Changelog

Усі помітні зміни цього пакета документуються тут.

Формат — [Keep a Changelog](https://keepachangelog.com/uk/1.1.0/), нумерація — [SemVer](https://semver.org/lang/uk/).

## [0.1.2] - 2026-05-26

### Added

- Додано canonical Stryker config для workspace-пакета, який створює правило `test`.

## [0.1.1] - 2026-05-22

### Added

- `coverage.js` — об'єднаний скрипт метрик: за один прогін збирає покриття коду (`bun test --coverage`, `cargo llvm-cov`) і мутаційне тестування (StrykerJS, `cargo mutants`), зводить усе в одну таблицю та записує `COVERAGE.md` у корінь монорепо для відстеження через git diff.
- `with-lock.js` — обгортка з ексклюзивним локом (`.coverage.lock`, PID + перевірка живучості, рекурсивна): усі команди метрик у `package.json` запускаються через неї, тож паралельні прогони неможливі.

## [0.1.0] - 2026-05-21

### Added

- Виокремлено `scripts/` в окремий workspace-пакет із власними залежностями `tinyglobby` і `gray-matter` (раніше були в кореневих devDependencies).
