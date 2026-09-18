# TRACE.md — Дорожная карта и Архитектурный трейс проекта

## 1. Общая концепция и разделение ответственности (Decoupling)

Архитектура системы строится на строгом разделении обязанностей:
1. **Frontend (UI & High-Level Features)** — **Flutter + Dart**:
   - Пользовательский интерфейс, темы, списки профилей и подписок.
   - Планировщик фоновых автообновлений подписок, балансировщик, выбор серверов.
   - Встроенный WebView или вызов системного браузера для отображения веб-панели **Metacubexd**.
   - Прямой вызов API ядра через **Dart FFI** (`cdylib` / C-ABI) без сетевых накладных расходов.

2. **Core Manager & Tooling (Independent Daemon/Library)** — **Rust (`core_manager`)**:
   - **Универсальный парсер (Universal Parsers)**: чтение 7 форматов (`v2ray`, `clash`, `sing-box`, `xray`, `happ`, `incy`, `throne`) с приведением к единой модели `ProfileConfig`.
   - **Утилита задержек и мета-информации (Latency & Metadata Tool)**:
     - ООП и плагинный дизайн на трейтах (`CheckerPlugin`, `MetadataProvider`).
     - Измерение TCP RTT, TLS Handshake, реального HTTP 204 запроса (Cloudflare/Google) и определение GeoIP/ISP.
     - Отдельный конфигурационный файл настроек приложения (таймауты, цели проверок, константы).
   - **Генератор Ultimate-конфига (Ultimate Config Template & Injector)**:
     - Master-шаблон JSON (маршрутизация, DNS, TUN, Clash API).
     - Инъекция узлов и параметров по путям/индексам вида `[outbounds][0]`.
   - **Супервайзер процесса ядра (Core Process Supervisor)**:
     - Асинхронный запуск процесса `singbox.exe` / `singbox` (`tokio::process`).
     - Захват логов `stdout`/`stderr` в ring-buffer и трансляция в Dart Streams.
     - Защита от сбоев, health checks и гарантированное освобождение интерфейса Wintun/TUN при завершении.
     - Интеграция с Clash API (порт `127.0.0.1:9090`, `external_ui: metacubexd`).

3. **Proxy Core (Data Plane Engine)** — **Go (`sing-box` / `libcore`)**:
   - Независимый бинарный файл, отвечающий исключительно за высокопроизводительное сетевое проксирование, шифрование и TUN/Wintun стек.
   - Взаимозаменяем: менеджер может запускать `sing-box`, `xray` или `mihomo` без пересборки менеджера.

---

## 2. Статус реализации задач

- [x] **Этап 1: Универсальный парсер конфигураций на Rust (`core_manager`)**
  - Реализованы модели `ProfileConfig`, `GroupConfig`, `AppConfig`, `TUNConfig`.
  - Реализованы парсеры: `V2RayParser`, `ClashParser`, `SingBoxParser`, `XrayParser`, `HappParser`, `IncyParser`, `ThroneParser`.
  - Реализован автодетектор форматов `detect_config_format()` и точка входа `parse_config()`.
  - Реализован конвертер в `sing-box` JSON (`to_singbox_json`).
  - Все тесты `cargo test` и `pytest` пройдены успешно (100%).

- [ ] **Этап 2: Утилита проверки задержек и метаданных (Latency & Metadata)**
  - Реализация трейтов плагинной архитектуры (`CheckerPlugin`, `MetadataProvider`).
  - Реализация TCP RTT Ping, TLS Handshake Ping, HTTP 204 Real Test.
  - Реализация резолвера GeoIP / ISP информации.
  - Разделение глобальных настроек приложения (таймауты, URL проверок).

- [ ] **Этап 3: Генератор Ultimate-конфига по путям/индексам**
  - Шаблон master-конфигурации для sing-box.
  - Механизм адресации и инъекции по путям `[path][index]`.

- [ ] **Этап 4: Асинхронный супервайзер процесса ядра и Metacubexd**
  - Управление процессом через `tokio::process`.
  - Потоковый сбор логов stdout/stderr.
  - Подключение и раздача статики Metacubexd через `clash_api.external_ui`.

- [ ] **Этап 5: C-ABI / Dart FFI слой для Flutter**
  - Экспорт функций для вызова из Dart.
  - Реализация событийных каналов (Streams) для логов и статуса подключения.

---

## 3. Документация существующего модуля Android (`app/`)
- Ведется поэтапный глубокий анализ кодовой базы `app/` с детальным описанием каждого файла в [`app/ARCHITECTURE.md`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/ARCHITECTURE.md).
