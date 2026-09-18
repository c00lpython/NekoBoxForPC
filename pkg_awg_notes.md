# Заметки по архитектуре AmneziaWG (pkg vs app)

## Архитектура Android-приложения (`app/`)
- Парсинг конфигурации `.conf` происходит в `io.nekohasekai.sagernet.fmt.Wireguard.kt`. Если в секции `[Interface]` присутствуют специфические ключи `jc`, `jmin`, `jmax`, `s1`, `s2`, `h1`, `h2`, `h3`, `h4`, то профиль классифицируется как AmneziaWG.
- При генерации конфигурации для sing-box (`ConfigBuilder.kt`) эти параметры преобразуются в JSON-объект с типом outbound `amneziawg` (так как используется форк sing-box с поддержкой AWG).

## Архитектура Go-бэкенда (`pkg/`)
- Парсинг конфигурации `.conf` реализован в `pkg/parsers/conf.go`. Логика обнаружения ключей (jc, jmin, и т.д.) перенесена корректно, профиль сохраняется с типом `models.TypeAmneziaWG` и заполненной структурой `AmneziaWGOptions`.
- **Проблема**: При генерации sing-box JSON в `pkg/compiler/config_builder.go` тип эндпоинта по умолчанию сбрасывается на `wireguard` и специфичные поля AWG игнорируются, если явно не установлен флаг `opts.UseAmneziaCore`. 
- При вызове через CLI (команды `parse`, `run`, `build`) в `cmd/cli/main.go` используется `opts := compiler.DefaultConfigBuilderOptions()`, в котором `UseAmneziaCore = false`. Из-за этого профили AWG теряют параметры обфускации и не могут подключиться (так как отправляют обычные WireGuard-пакеты).

## Решение
В Go-пакете нужно убрать зависимость от флага `UseAmneziaCore` при обработке профиля типа `TypeAmneziaWG`. Раз профиль уже распарсился как AWG, мы должны генерировать outbound с типом `amneziawg` и всеми необходимыми параметрами обфускации. Флаг `UseAmneziaCore` можно вообще удалить или игнорировать в этом контексте, так как используемая сборка `sing-box` имеет встроенную поддержку `amneziawg`.
