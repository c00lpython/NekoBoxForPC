# Спецификация CLI NekoBoxPlusForPC (`nbpfpc`) и Schema Mapping

Документ описывает архитектуру командной строки, интерактивный и моментальный конструкторы конфигураций и групп, механизм хранения состояния и семантический маппинг схем данных между сущностями CLI/Storage и ядром `sing-box`.

---

## 1. Команды CLI (`nbpfpc`)

### 1.1. Конструктор конфигураций
* `nbpfpc new [target] [--group <name>] [--instant <json> | -i <json>]`
  - Создание нового профиля прокси.
  - **Моментальный режим (`--instant`)**: принимает полный JSON-блок в формате outbounds ядра `sing-box` (или путь к файлу JSON), либо полный конфиг (извлекает первый рабочий прокси-аутбаунд).
    ```bash
    nbpfpc new --group Servers --instant '{"type":"vless","tag":"node-nl","server":"94.130.1.2","server_port":443,"uuid":"..."}'
    ```
  - **Диалоговый режим (Wizard)**: пошаговый опрос полей (`protocol`, `name/tag`, `server`, `port`, `uuid/password`, `security/tls`, `sni`) со стандартными подсказками (`443, 80, 8080, 2080, 1080`, протоколы, типы TLS).
    - Переход к следующему шагу: `Enter` (применяет текущее/дефолтное значение).
    - Возврат к предыдущему шагу (Prev): ввод `<` или `prev`.
* `nbpfpc edit <group.config> [--instant <json> | -i <json>]`
  - Редактирование существующего профиля. В диалоговом режиме отображается текущее значение каждого поля в подсказке `[текущее: value]` с возможностью перехода назад `<` и подтверждения по `Enter`.
* `nbpfpc delete <group.config | config>`
  - Удаление профиля из группы с автоматическим сбросом `active_picked`, если удаленный профиль был выбран.
* `nbpfpc pick <group.config | config>`
  - Выбор профиля в качестве активного по умолчанию для запуска ядра.
* `nbpfpc unpick`
  - Сброс активного выбора.

### 1.2. Конструктор групп и подписок
* `nbpfpc newgroup [name] [--sub <URL>] [--instant <json> | -i <json>]`
  - Создание новой группы: `manual` (ручная) или `sub` (подписка).
  - Быстрое создание подписки: `nbpfpc newgroup MySub --sub "https://provider.com/sub"` (автоматически скачивает и парсит ноды).
  - В моментальном режиме: JSON с полями `name`, `group_type`, `subscription_url`, `auto_update_minutes`, `client_imitation`, `hwid`.
  - В интерактивном режиме: пошаговый опрос с подсказками интервалов (0, 30, 60, 120, 1440 минут) и клиентов (`v2rayN`, `ClashMeta`, `sing-box`, `Happ`, `v2raytun`, `Incy`).
* `nbpfpc update [GROUPNAME]`
  - Обновление конфигурации подписки по сети: выполнение HTTP GET запроса с фингерпринтом клиента и HWID, автоматическая распаковка Base64/YAML/JSON, обновление списка узлов и сохранение метаинформации трафика `Subscription-Userinfo`.
  - Если `GROUPNAME` опущен — обновляются все группы типа `sub`.
* `nbpfpc sub-info <GROUPNAME>`
  - Просмотр полной информации о подписке: использованный трафик (отправлено / скачано), суммарный лимит, процент расхода, дата и время истечения срока, статус HWID, User-Agent и время последнего обновления.
* `nbpfpc editgroup <GROUPNAME> [--instant <json> | -i <json>]`
  - Редактирование параметров группы.
* `nbpfpc deletegroup <GROUPNAME>`
  - Удаление группы со всеми входящими в нее профилями.

### 1.3. Управление жизненным циклом ядра
* `nbpfpc run [group.config | URI | config.json] [--core <path>] [--tun]`
  - Запуск сессии ядра `sing-box`.
  - **Режим по умолчанию (без TUN, `--tun=false`)**:
    - Не требует прав администратора и не вмешивается в системные адаптеры Wintun.
    - Ядро поднимает локальный смешанный прокси (Mixed SOCKS5 / HTTP) на `127.0.0.1:2080`.
    - Поднимается веб-панель Clash API на `http://127.0.0.1:9090/ui` для переключения узлов и мониторинга.
  - **Режим системного TUN (`--tun`)**:
    - Направляет весь сетевой трафик ОС через прокси-интерфейс TUN.
  - Порядок разрешения цели (если аргумент не указан):
    1. `active_picked` (профиль, выбранный командой `pick`).
    2. `last_run` (профиль, запускавшийся в прошлый раз).
    3. Первый доступный профиль в первой непустой группе.
  - Поддерживает Ctrl+C для корректной остановки и очистки ресурсов.
* `nbpfpc stop`
  - Аварийная или фоновая остановка активных процессов ядра sing-box (`taskkill /F /IM singbox.exe` на Windows / `pkill` на Unix) и освобождение сетевых адаптеров Wintun/TUN.
* `nbpfpc list`
  - Отображение иерархии групп и конфигов, протоколов, хостов, задержек и маркеров статуса:
    ```text
    ============================================================
             Хранилище конфигураций NekoBoxPlusForPC            
    ============================================================
     [АКТИВНЫЙ ВЫБОР (PICKED)]: Servers.node-nl
    ------------------------------------------------------------
    📁 Группа: Servers (Ручная)
       * [PICKED] node-nl          | vless      | 94.130.1.2:443         |    2 ms
          node-de                  | trojan     | 1.2.3.4:8443           |    4 ms
    ============================================================
    ```

### 1.4. Проверка задержек (Ping) и диапазоны
* `nbpfpc ping [selector]`
  - Проверка задержек и автосохранение результатов в `store.json`.
  - Варианты селекторов:
    - Все профили: `nbpfpc ping` (без аргументов)
    - Одиночный узел: `nbpfpc ping Servers.node-nl`
    - Вся группа: `nbpfpc ping Servers`
    - Алфавитные диапазоны имен узлов: `nbpfpc ping "[a-c],[h-z]"`
    - Инверсия диапазона/выбора: `nbpfpc ping "invert [a-c],[h-z]"` (выбирает все узлы, которые НЕ попадают в указанные диапазоны).
    - Инверсия группы: `nbpfpc ping "invert Servers"` (выбирает все узлы кроме указанной группы).

---

## 2. Schema Mapping Manifesto (Маппинг схем данных)

### 2.1. Исходная схема (Storage / CLI Input)
```yaml
StoredGroup:
  name: string (primary key)
  group_type: string ("manual" | "sub")
  subscription_url: optional string
  auto_update_minutes: u32
  hwid: bool
  client_imitation: string (e.g. "v2rayN", "Clash", "Shadowrocket")
  front_proxy: optional string
  outbound_proxy: optional string
  profiles: list<StoredProfile>

StoredProfile:
  id: string (uuid)
  name: string
  protocol: string ("vless" | "vmess" | "trojan" | "shadowsocks" | "hysteria2" | "wireguard" | ...)
  server: string (ip or fqdn)
  server_port: u16
  settings: json_map (protocol-specific parameters: uuid, password, method, flow, tls, sni, pbk, sid)
  tag: string
  last_ping_ms: optional u64
```

### 2.2. Целевая схема (`sing-box` Outbound Object)
```yaml
SingBoxOutbound:
  type: string ("vless", "vmess", "trojan", "shadowsocks", "hysteria2", "wireguard", "direct", "block")
  tag: string (unique outbound tag)
  server: string
  server_port: u16
  uuid: optional string
  password: optional string
  method: optional string
  flow: optional string
  tls: optional object {
    enabled: bool,
    server_name: string,
    reality: optional object {
      enabled: bool,
      public_key: string,
      short_id: string
    }
  }
```

### 2.3. Полевой маппинг (Field-Level Mapping Table)

| Целевое поле (sing-box) | Исходное поле (CLI / StoredProfile) | Тип маппинга | Описание и трансформация |
| :--- | :--- | :--- | :--- |
| `type` | `protocol` | Direct / Normalized | Приведение протокола к нижнему регистру (`vless`, `vmess`, `trojan`, `shadowsocks`, `hysteria2`) |
| `tag` | `tag` / `name` | Derived | Уникальный тег узла (если не задан, формируется `proxy-{id[..8]}`) |
| `server` | `server` | Direct / Cleaned | Хост или IP-адрес, очищенный от пробелов |
| `server_port` | `server_port` | Direct / Cast | Целое число от 1 до 65535, дефолт `443` |
| `uuid` | `settings.uuid` | Direct | Для VLESS / VMess (строка UUID) |
| `password` | `settings.password` | Direct | Для Trojan / Shadowsocks / Hysteria2 |
| `method` | `settings.method` | Defaulted | Для Shadowsocks (по умолчанию `aes-256-gcm` или `2022-blake3-aes-128-gcm`) |
| `flow` | `settings.flow` | Optional Direct | Для VLESS (`xtls-rprx-vision`) |
| `tls.enabled` | `settings.tls` / `settings.security` | Derived | `true` если security равен `tls` или `reality`, либо флаг `tls == true` |
| `tls.server_name`| `settings.sni` / `settings.host` | Coalesced | Если `sni` отсутствует, fallback на `host`, затем на `server` |
| `tls.reality` | `settings.pbk`, `settings.sid` | Derived | Если указан `pbk` (public key), формируется объект `reality: { enabled: true, public_key, short_id }` |

---

## 3. Архитектура хранилища состояния (`ConfigStore`)

Файл хранилища располагается по пути `.runtime/store.json`.
Структура:
```json
{
  "active_picked": "Default.MyProxy",
  "last_run": "Default.MyProxy",
  "groups": [
    {
      "name": "Default",
      "group_type": "manual",
      "subscription_url": null,
      "auto_update_minutes": 0,
      "hwid": false,
      "client_imitation": "v2rayN",
      "front_proxy": null,
      "outbound_proxy": null,
      "profiles": [ ... ]
    }
  ]
}
```
