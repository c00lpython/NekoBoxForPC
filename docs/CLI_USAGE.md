# NekoBoxPlusForPC (nbpfpc) — Руководство по установке и использованию

`nbpfpc` — это консольный менеджер и супервайзер прокси-ядра sing-box, написанный на Rust. Утилита предоставляет универсальный парсер конфигураций 7 форматов, изолированный запуск без TUN (SOCKS5/HTTP на порту 20808 + Clash API на порту 9090), параллельный сортированный пинг, интерактивный TUI-пейджер, маршрутизацию, локальное хранилище гео-баз (GeoIP/GeoSite) и гранулярное резервное копирование (`backup`/`recovery`).

---

## 1. Установка и сборка

### 1.1. Системные требования
- **ОС:** Windows 10/11, Linux (x86_64, arm64), macOS.
- **Rust & Cargo:** версия 1.75+ (установка через [rustup.rs](https://rustup.rs/)).

### 1.2. Сборка из исходников
В корне репозитория выполните:

```powershell
# Сборка оптимизированного релизного бинарника
cargo build --release --manifest-path core_manager/Cargo.toml

# Копирование бинарника в рабочую директорию
Copy-Item core_manager/target/release/nbpfpc.exe ./nbpfpc.exe
```

Для сборки в режиме отладки:
```powershell
cargo build --manifest-path core_manager/Cargo.toml
Copy-Item core_manager/target/debug/nbpfpc.exe ./nbpfpc.exe
```

---

## 2. Общие принципы вызова команд

Все команды поддерживают **два равнозначных формата вызова**:
1. Стандартный: `nbpfpc <command> [args]` (например, `.\nbpfpc.exe ping`)
2. С префиксом дефиса: `nbpfpc -<command> [args]` (например, `.\nbpfpc.exe -ping`)

Все настройки, группы, профили, правила и гео-базы сохраняются локально в папке `.runtime/` (`store.json` и `geo/`).

---

## 3. Справочник команд

### 3.1. Справка и документация

| Команда | Описание |
| :--- | :--- |
| `nbpfpc help` / `nbpfpc -help` | Общее руководство по всем разделам программы |
| `nbpfpc help <COMMAND>` | Подробная справка по конкретной команде (аргументы, опции) |

**Примеры:**
```powershell
.\nbpfpc.exe -help
.\nbpfpc.exe help ping
.\nbpfpc.exe help backup
.\nbpfpc.exe help recovery
```

---

### 3.2. Конфигурации и группы (CRUD)

| Команда | Описание |
| :--- | :--- |
| `nbpfpc list [GROUP]` | Интерактивный терминальный пейджер с автоопределением высоты экрана и сортировкой |
| `nbpfpc newconfig [NAME] [OPTIONS]` | Создание узла (`--type <usual\|dialog>`, протоколы: `vless`, `vmess`, `trojan`, `ss`, `hysteria2`, `wireguard`, `tuic`, `byedpi`, `masterdnsvpn`, `proxychain`, `balancer`) |
| `nbpfpc delconfig <TARGET>` | Удаление узла по имени (`Group.node`) или индексу |
| `nbpfpc editconfig <TARGET> [OPTIONS]` | Редактирование существующего узла (`--type <usual\|dialog>`) |
| `nbpfpc pick [TARGET]` | Выбор активного узла для запуска или запуск интерактивного смарт-меню |
| `nbpfpc unpick` | Сброс активного выбора |
| `nbpfpc select <SELECTOR>` | Выборка узлов по номерам, именам или диапазонам (`[a-c]`, `1,3,5`) |
| `nbpfpc newgroup <NAME> [OPTIONS]` | Создание ручной группы (`LIST`) или подписки (`SUB`) |
| `nbpfpc delgroup <NAME>` | Удаление группы |
| `nbpfpc editgroup <NAME>` | Редактирование параметров группы |
| `nbpfpc update [GROUP]` | Обновление конфигурации подписки по сети со счетчиком узлов |
| `nbpfpc subinfo <GROUP>` | Просмотр остатка трафика и срока действия подписки |

**Примеры:**
```powershell
# Просмотр всех узлов в интерактивном TUI-пейджере
# Навигация и горячие клавиши (поддерживают русскую и английскую раскладки):
#   ← / →, [ / ]                : листание ровно по 1 странице (±1)
#   PgUp / PgDn, A / D, Ф / В   : быстрое листание по 2 страницы (±2)
#   ↑ / ↓, W / S, Ц / Ы         : переключение режима сортировки:
#                                 По умолчанию (номер) -> Пинг (быстрые) -> Пинг (медленные) -> Имя -> Группа -> Протокол
#   Home / End                  : переход на первую / последнюю страницу
#   /, Ctrl+F, Ctrl+А           : поиск и фильтрация по названию/протоколу/группе
#   Q, Й, Esc                   : выход из пейджера
.\nbpfpc.exe -list

# Добавление профиля подключения через интерактивный диалог:
.\nbpfpc.exe -newconfig MyNode --type dialog --protocol vless

# Создание узла ByeDPI:
.\nbpfpc.exe -newconfig ByeDPI-Node --protocol byedpi --server 127.0.0.1 --port 1080

# Создание узла MasterDnsVPN:
.\nbpfpc.exe -newconfig DnsVPN-Node --protocol masterdnsvpn --server 8.8.8.8 --port 53

# Добавление группы подписки с автообновлением раз в 12 часов
.\nbpfpc.exe -newgroup MySub --group-type SUB --sub "https://example.com/sub" --autoupdate 12h

# Выбор активного узла
.\nbpfpc.exe -pick "ALLVPN.🇷🇺 Soda VPN | Россия"
.\nbpfpc.exe -unpick
```

---

### 3.3. Сеть, тестирование и запуск ядра

| Команда | Описание |
| :--- | :--- |
| `nbpfpc run [TARGET] [OPTIONS]` | Запуск прокси-сессии sing-box без TUN (Mixed: `127.0.0.1:20808`, Clash UI: `9090`) |
| `nbpfpc ping [TARGET] [OPTIONS]` | Параллельное измерение задержек с сохранением в store.json, сортировкой и режимами вывода |
| `nbpfpc stop` | Остановка запущенных процессов ядра sing-box |
| `nbpfpc logs [-l N] [-e]` | Просмотр логов последней сессии (`-e` — только ошибки) |
| `nbpfpc newbalancer <GROUP> [NAME]` | Создание профиля-балансировщика `urltest` по узлам группы |
| `nbpfpc newchain <GROUP> <NAME> <NODES>` | Создание последовательной цепочки прокси (detour) |

**Опции тестирования (`ping` / `-ping`):**
- `-m, --mode <table|list|pager>`:
  - `table` (по умолчанию): детальная цветная таблица (номер, задержка, группа, узел, хост:порт).
  - `list`: отображение в стиле команды `list` (`1.(GROUP) [PICKED] NAME | PROTOCOL | XX ms`).
  - `pager`: запуск полноэкранного интерактивного TUI-пейджера по свежим результатам замера.
- `-s, --sort <fastest|slowest|name|group|original>`:
  - `fastest` (или `asc`, по умолчанию): от самых быстрых к медленным, таймауты в конце.
  - `slowest` (или `desc`): от самых медленных к быстрым, таймауты в конце.
  - `name` (или `alpha`): по алфавитному порядку названий конфигураций.
  - `group`: группировка по названию группы, затем по алфавиту имени.
  - `original`: исходный порядок следования в хранилище / селекторе.
- `--save` / `--no-save`: сохранять ли измеренные миллисекунды в LocalStorage (`store.json`). По умолчанию сохраняются автоматически через быстрый пакетный коммит.
- `--export <FILE>`: путь для выгрузки подробного отчета тестирования в структурированный JSON (время замера, средний пинг, количество таймаутов, полный список узлов).
- `-t, --threads <N>`: число параллельных потоков тестирования (по умолчанию ядра/2, от 2 до 30).

**Опции запуска (`run`):**
- `--direct`: локальный SOCKS5 без удаленного прокси.
- `--chain <nodes>`: запуск цепочки узлов через запятую.
- `--balancer <selector>`: запуск балансировщика по диапазону или группе.
- `--chain-balancer <front,group>`: фронт-узел + балансировщик группы.
- `--log-level <trace|debug|info|warn|error>`: уровень логирования ядра.
- `--no-log`: фоновый запуск без вывода логов ядра в консоль.

**Примеры:**
```powershell
# Тест группы ALLVPN в режиме компактного списка:
.\nbpfpc.exe -ping ALLVPN -mode list

# Тест с сортировкой от самых медленных узлов к быстрым:
.\nbpfpc.exe ping ALLVPN -mode list -sort slowest

# Запуск интерактивного пейджера по свежезамеренным узлам:
.\nbpfpc.exe ping ALLVPN -mode pager

# Тест без перезаписи задержек в store.json и с экспортом отчета в JSON:
.\nbpfpc.exe ping ALLVPN --no-save --export .\reports\ping_allvpn.json

# Запуск активного узла без захвата сетевых адаптеров (TUN)
.\nbpfpc.exe -run

# Просмотр последних 30 строк логов сессии
.\nbpfpc.exe -logs -l 30
```

---

#### 3.4. Правила маршрутизации (Routing Rules)

| Команда | Описание |
| :--- | :--- |
| `nbpfpc ruleslist` / `-ruleslist` | Таблица всех правил маршрутизации со статусом `[ВКЛ]`/`[ВЫКЛ]` |
| `nbpfpc newrule [NAME] [ACTION] ...` | Создание правила (интерактивный TUI-редактор с окном автодополнений SITE и IP или через аргументы) |
| `nbpfpc editrule [NAME] [OPTIONS]` | Редактирование существующего правила (интерактивно или по флагам) |
| `nbpfpc delrule <NAME>` | Удаление правила |
| `nbpfpc moverule <NAME> <POS>` | Изменение приоритета правила в цепочке маршрутизации |
| `nbpfpc switchrule <NAME> [on\|off]` | Включение или выключение правила |

**Доступные действия (`ACTION`):** `direct`, `proxy`, `block`, `dns`.

**Автодополнения и префиксы в TUI-окне:**
- **SITE (домены):** `geosite:category-ads-all`, `geosite:google`, `geosite:youtube`, `geosite:telegram`, `geosite:openai`, `geosite:ru`, `geosite:cn`, `domain:`, `domain_suffix:`, `domain_regex:`, `rule_set:`.
- **IP (адреса):** `geoip:private`, `geoip:ru`, `geoip:telegram`, `ip_cidr:10.0.0.0/8`, `ip_cidr:192.168.0.0/16`, `ip_cidr:172.16.0.0/12`, `rule_set:`.

**Примеры:**
```powershell
# Запуск интерактивного TUI-мастера создания правила с окном автодополнений:
.\nbpfpc.exe -newrule

# Запуск интерактивного TUI-редактора существующего правила:
.\nbpfpc.exe -editrule MyRule

# Быстрое создание правила блокировки рекламы и локальных сетей через аргументы:
.\nbpfpc.exe -newrule BlockAds block --site geosite:category-ads-all --ip geoip:private

# Модификация правила через флаги:
.\nbpfpc.exe -editrule BlockAds --action proxy --site geosite:google,domain:example.com

# Временно отключить правило
.\nbpfpc.exe -switchrule BlockAds off

# Посмотреть таблицу правил
.\nbpfpc.exe -ruleslist

# Удалить правило
.\nbpfpc.exe -delrule BlockAds
```

---

### 3.5. Резервное копирование и восстановление (Backup & Recovery)

Система бэкапов позволяет создавать переносимые JSON-снимки состояния и восстанавливать их с выборочным контролем категорий:

#### Команда `backup`
```text
nbpfpc backup <NAME> [DESTINATIONPATH] [CONFIGS] [ROUTES] [SETTINGS]
```
- `NAME`: имя резервной копии.
- `DESTINATIONPATH`: папка для сохранения (по умолчанию `.\backups`).
- `CONFIGS`: сохранять ли группы и профили: `true`/`false` (дефолт: `true`).
- `ROUTES`: сохранять ли правила маршрутизации: `true`/`false` (дефолт: `true`).
- `SETTINGS`: сохранять ли настройки: `true`/`false` (дефолт: `true`).

#### Команда `recovery`
```text
nbpfpc recovery <PATH> [CONFIGS] [ROUTES] [SETTINGS] [OPTIONS]
```
- `PATH`: путь к файлу резервной копии (`.\backups\<NAME>.json`).
- `CONFIGS`: применить ли конфигурации: `true`/`false` (дефолт: `true`).
- `ROUTES`: применить ли правила маршрутизации: `true`/`false` (дефолт: `true`).
- `SETTINGS`: применить ли настройки: `true`/`false` (дефолт: `true`).
- `-m, --mode <merge|hard>`:
  - `merge` (по умолчанию): интеллектуальное слияние без удаления существующих данных (новые узлы и правила добавляются, существующие обновляются, созданные пользователем локальные группы и правила сохраняются).
  - `hard`: полная замена выбранных компонентов данными из бэкапа (аналог `git reset --hard`).
- `-y, --yes`: пропустить интерактивное окно подтверждения и применить восстановление сразу.

Перед применением `recovery` автоматически выводит интерактивное окно подтверждения с детальной сводкой:
- список восстанавливаемых групп и профилей (пометка `[НОВАЯ ГРУППА]` или `[СЛИЯНИЕ]`);
- список правил маршрутизации (пометка `[НОВОЕ]` или `[ОБНОВЛЕНИЕ]`);
- список изменений в настройках и базах Geo.

**Примеры:**
```powershell
# Полный бэкап в .\backups\MyBackup.json
.\nbpfpc.exe -backup MyBackup .\backups true true true

# Бэкап только правил маршрутизации
.\nbpfpc.exe backup OnlyRouting .\backups false true false

### 3.6. Локальное хранилище гео-данных (GeoIP / GeoSite)

Гео-файлы баз данных IP-адресов и доменов сохраняются в защищенном локальном хранилище `.runtime/geo/`, автоматически индексируются в реестре хранилища с расчетом SHA256-хеша и подключаются к ядру sing-box при сборке конфигураций:

| Команда | Описание |
| :--- | :--- |
| `nbpfpc geolist` | Список всех сохраненных гео-файлов, их размер, хеш и источник |
| `nbpfpc geoupdate [NAME]` | Скачивание и обновление свежих баз `geoip.db` и `geosite.db` по сети |
| `nbpfpc geofetch <URL\|PATH> [NAME]` | Импорт гео-файла из интернета или локального пути в хранилище |
| `nbpfpc geodel <NAME>` | Удаление гео-файла из реестра и диска |

**Примеры:**
```powershell
# Список сохраненных гео-файлов
.\nbpfpc.exe -geolist

# Скачивание актуальных баз SagerNet GeoIP и GeoSite
.\nbpfpc.exe -geoupdate all

# Импорт локального файла или ссылки
.\nbpfpc.exe -geofetch "https://custom-repo/geosite-custom.db" my-geosite.db --geo-type geosite
.\nbpfpc.exe -geofetch "C:\Downloads\geoip.db" geoip.db --geo-type geoip

# Удаление гео-файла
.\nbpfpc.exe -geodel my-geosite.db
```

---

### 3.7. Интерактивные настройки (`settings`)

Команда `nbpfpc settings` (или `nbpfpc -settings`) открывает интерактивное TUI-меню терминала с 7 категориями:
1. `Interface` — тема (dark/light), иконки, отображение стран.
2. `Connection` — режим подключения (Proxy/VPN), стек (mixed/gVisor/system).
3. `Core` — протокол мультиплексирования (h2mux/yamux), лимит памяти, валидация сертификатов.
4. `DNS` — FakeDNS, Direct DNS, Proxy DNS, IPv6, гео-ресурсы.
5. `Tests` — URL проверки задержек, таймауты, тип тестов (HTTP/TCP/RTT).
6. `Logging` — уровень логирования (`trace`/`debug`/`info`), путь сохранения логов.
7. `Other` — uTLS фингерпринт (`chrome`/`firefox`), автообновление подписок и ресурсов правил.

Управление: стрелки `Вверх`/`Вниз` для выбора, `Enter` для изменения значения, `Q` для выхода с сохранением.

---

### 3.8. Встроенные ассеты и готовые шаблоны (`assets`)

В `nbpfpc` включены готовые предустановленные профили и группы:
1. **Группа "WARP":**
   - **Cloudflare WARP (AWG):** AmneziaWG 2.0 с заготовленными параметрами маскировки пакетов (`Jc=4`, `Jmin=40`, `Jmax=70`, `S1=0`, `S2=0`, `H1`-`H4`).
   - **Cloudflare WARP (MASQUE):** VLESS / MASQUE over HTTP/3 CONNECT-UDP через Anycast-сеть Cloudflare.
2. **Группа "Goida Group":**
   - Бесплатная автообновляемая подписка (Free Sub, интервал обновления 12 часов) со стартовыми нодами прямого обхода (`🇷🇺 GOIDA Freedom (Bypass)`) и резервного проксирования (`🌐 GOIDA Cloudflare Fallback`).

| Команда | Описание |
| :--- | :--- |
| `nbpfpc assets install` / `-assets` | Установка групп `WARP` и `Goida Group` в локальное хранилище |
| `nbpfpc assets list` | Просмотр состава доступных встроенных ассетов |

```powershell
# Просмотр состава ассетов
.\nbpfpc.exe assets list

# Установка готовых ассетов WARP и Goida Group
.\nbpfpc.exe assets install

# Тестирование задержек группы WARP
.\nbpfpc.exe ping WARP -mode list
```
