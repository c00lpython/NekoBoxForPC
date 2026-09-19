use clap::{Parser, Subcommand};
use core_manager::latency::CheckResult;
use core_manager::{
    build_group_instant, build_group_interactive, build_profile_instant,
    detect_config_format, parse_config, ConfigStore,
    CoreSupervisor, ParsedSelector, PluginRegistry, ProfileConfig, ProtocolType,
    StoredGroup, StoredProfile, SubscriptionHandler, SubscriptionUserInfo, UltimateConfigBuilder,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "nbpfpc")]
#[command(disable_help_subcommand = true)]
#[command(
    about = "NekoBoxPlusForPC CLI — менеджер прокси ядра, универсальный конструктор и селектор",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Подробная справка о команде или о программе в целом
    #[command(name = "help")]
    Help {
        /// Имя команды для получения подробной справки
        command_name: Option<String>,
    },

    /// Создание нового профиля (обычный из JSON/URI или диалоговый)
    #[command(name = "newconfig", alias = "new")]
    NewConfig {
        /// Имя профиля (например "my-proxy" или "Default.my-proxy")
        name: Option<String>,
        /// Протокол узла (vless, vmess, trojan, ss, hysteria2, wireguard, amneziawg, tuic, ssh, byedpi, masterdnsvpn, proxychain, balancer, direct)
        #[arg(short, long)]
        protocol: Option<String>,
        /// Тип создания: usual (обычный) или dialog (интерактивный диалог)
        #[arg(long = "type", alias = "config_type", default_value = "usual")]
        config_type: String,
        /// Моментальный JSON-конфиг аутбаунда или URI-ссылка
        #[arg(short, long)]
        instant: Option<String>,
        /// Имя целевой группы (по умолчанию "Default")
        #[arg(short, long, default_value = "Default")]
        group: String,
    },

    /// Удаление профиля из хранилища по имени или индексу
    #[command(name = "delconfig", alias = "delete")]
    DelConfig {
        /// Имя узла ("Default.node", "node") или его номер/индекс в списке
        target: String,
    },

    /// Редактирование существующего профиля по имени или индексу
    #[command(name = "editconfig", alias = "edit")]
    EditConfig {
        /// Имя узла ("Default.node", "node") или его номер в списке
        target: String,
        /// Тип редактирования: usual или dialog
        #[arg(long = "type", alias = "config_type", default_value = "dialog")]
        config_type: String,
        /// Моментальный новый JSON аутбаунда или путь к файлу
        #[arg(short, long)]
        instant: Option<String>,
    },

    /// Постраничный просмотр всех узлов или узлов указанной группы с пагинацией и поиском
    #[command(name = "list")]
    List {
        /// Имя группы (например "ALLVPN" или "-ALLVPN"). Если не указано — выводятся все узлы.
        group: Option<String>,
    },

    /// Выбор узлов по перечислению или диапазонам (например "[a-c]", "1,3,5")
    #[command(name = "select")]
    Select {
        /// Селектор: "group", "[a-c]", "1,3,5", "invert [a-c]"
        selector: String,
    },

    /// Выбор активного профиля для ядра (по имени или запуск smart-меню)
    #[command(name = "pick")]
    Pick {
        /// Имя узла ("Group.node", "node") или его индекс
        target: Option<String>,
    },

    /// Снятие выбора активного профиля
    #[command(name = "unpick")]
    Unpick,

    /// Создание новой группы узлов или подписки
    #[command(name = "newgroup")]
    NewGroup {
        /// Имя новой группы
        name: Option<String>,
        /// Тип группы: LIST (ручная) или SUB (подписка)
        #[arg(long, default_value = "LIST")]
        group_type: String,
        /// URL подписки (обязателен, если тип SUB)
        #[arg(long)]
        sub: Option<String>,
        /// Период автообновления подписки (например "30m", "12h", "1d")
        #[arg(long)]
        autoupdate: Option<String>,
        /// Моментальный JSON группы
        #[arg(short, long)]
        instant: Option<String>,
    },

    /// Удаление группы узлов
    #[command(name = "delgroup", alias = "deletegroup")]
    DelGroup {
        /// Имя удаляемой группы
        name: String,
    },

    /// Редактирование параметров группы
    #[command(name = "editgroup")]
    EditGroup {
        /// Имя группы
        name: String,
        /// Тип редактирования: usual или dialog
        #[arg(long, default_value = "dialog")]
        config_type: String,
        /// Моментальный JSON группы
        #[arg(short, long)]
        instant: Option<String>,
    },

    /// Запуск сессии sing-box с выбранным узлом, группой или балансировщиком
    #[command(name = "run")]
    Run {
        /// Имя узла ("Group.node"), URI ссылка или пусто для активного (picked)
        target: Option<String>,
        /// Прямой локальный SOCKS5 без удаленного прокси (direct outbound)
        #[arg(long, default_value_t = false)]
        direct: bool,
        /// Запуск цепочки прокси (узлы через запятую)
        #[arg(long, allow_hyphen_values = true)]
        chain: Option<String>,
        /// Запуск балансировщика (диапазон "-list [a-c]" или группа "-group UserSub")
        #[arg(long, allow_hyphen_values = true)]
        balancer: Option<String>,
        /// Цепочка с балансировщиком: фронт-узел и группа ("front_node,group_name")
        #[arg(long, allow_hyphen_values = true)]
        chain_balancer: Option<String>,
        /// Путь к бинарнику ядра sing-box
        #[arg(short, long)]
        core: Option<PathBuf>,
        /// Включить сетевой интерфейс TUN
        #[arg(long, default_value_t = false)]
        tun: bool,
        /// Уровень логирования sing-box (trace, debug, info, warn, error)
        #[arg(long, default_value = "trace")]
        log_level: String,
        /// Отключить вывод логов ядра в консоль
        #[arg(long, default_value_t = false)]
        no_log: bool,
    },

    /// Параллельное тестирование задержек с сортировкой и индикатором прогресса
    #[command(name = "ping")]
    Ping {
        /// Селектор узлов: имя узла, имя группы ("ALLVPN"), диапазон "[a-c]" или пусто (все)
        #[arg(default_value = "", allow_hyphen_values = true)]
        selector: String,
        /// Количество параллельных потоков (по умолчанию доступные ядра / 2)
        #[arg(short, long)]
        threads: Option<usize>,
        /// Режим отображения результатов: table (таблица), list (список), pager (интерактивный пейджер)
        #[arg(short, long, default_value = "table")]
        mode: String,
        /// Критерий сортировки: fastest (asc), slowest (desc), name (по алфавиту), group (по группам), original (исходный порядок)
        #[arg(short, long, default_value = "fastest")]
        sort: String,
        /// Сохранять ли результаты замера задержек в LocalStorage (store.json)
        #[arg(long, default_value_t = true)]
        save: bool,
        /// Отключить сохранение результатов в LocalStorage
        #[arg(long, default_value_t = false)]
        no_save: bool,
        /// Путь для экспорта отчета в файл (JSON или TXT)
        #[arg(long)]
        export: Option<PathBuf>,
    },

    /// Добавление правила маршрутизации (newrule [NAME] [ACTION] [OPTIONS])
    #[command(name = "newrule")]
    NewRule {
        /// Имя правила (если не указано — открывается интерактивный TUI-редактор)
        name: Option<String>,
        /// Действие: direct, proxy, block, dns
        action: Option<String>,
        /// Список доменов через запятую или префикс "SITE:domain1,domain2"
        #[arg(short, long)]
        site: Option<String>,
        /// Список CIDR через запятую или префикс "IP:cidr1,cidr2"
        #[arg(short, long)]
        ip: Option<String>,
        /// Необязательные позиционные аргументы (например SITE:google.com IP:1.1.1.1/32)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },

    /// Редактирование правила маршрутизации (editrule [NAME] [OPTIONS])
    #[command(name = "editrule")]
    EditRule {
        /// Имя правила для редактирования
        name: Option<String>,
        /// Новое действие: direct, proxy, block, dns
        #[arg(short, long)]
        action: Option<String>,
        /// Список доменов через запятую
        #[arg(short, long)]
        site: Option<String>,
        /// Список CIDR через запятую
        #[arg(short, long)]
        ip: Option<String>,
        /// Необязательные позиционные аргументы
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },

    /// Удаление правила маршрутизации
    #[command(name = "delrule")]
    DelRule {
        /// Имя правила
        name: String,
    },

    /// Список всех правил маршрутизации с индексами и статусами
    #[command(name = "ruleslist", alias = "rules")]
    RulesList,

    /// Перемещение правила на новую позицию в цепочке маршрутизации
    #[command(name = "moverule")]
    MoveRule {
        /// Имя перемещаемого правила
        name: String,
        /// Новая позиция правила (1-based индекс)
        newpos: usize,
    },

    /// Переключение статуса правила (включить/выключить)
    #[command(name = "switchrule")]
    SwitchRule {
        /// Имя правила
        name: String,
        /// Желаемое состояние: on, off, true, false (если не указано — инвертировать)
        state: Option<String>,
    },

    /// Интерактивный терминальный TUI для просмотра и редактирования настроек
    #[command(name = "settings")]
    Settings,

    /// Просмотр логов сессий ядра
    #[command(name = "logs")]
    Logs {
        /// Количество строк для вывода с конца
        #[arg(short, long, default_value_t = 50)]
        lines: usize,
        /// Очистить каталог логов
        #[arg(long, default_value_t = false)]
        clear: bool,
        /// Показывать только ошибки (ERROR, FATAL, failed, STDERR)
        #[arg(short, long, default_value_t = false)]
        errors: bool,
        /// Имя конкретного лог-файла в папке logs/
        name: Option<String>,
    },

    /// Остановка активных экземпляров ядра sing-box
    #[command(name = "stop")]
    Stop,

    /// Обновление конфигурации подписки по сети
    #[command(name = "update")]
    Update {
        /// Имя группы подписки (если не указано — обновляются все подписки)
        group: Option<String>,
    },

    /// Просмотр метаинформации и остатка трафика подписки
    #[command(name = "subinfo")]
    SubInfo {
        /// Имя группы подписки
        group: String,
    },

    /// Создать профиль-балансировщик urltest в группе
    #[command(name = "newbalancer")]
    NewBalancer {
        /// Имя группы
        group: String,
        /// Имя профиля балансировщика
        #[arg(default_value = "Auto-Best")]
        name: String,
    },

    /// Создать профиль-цепочку прокси в группе
    #[command(name = "newchain")]
    NewChain {
        /// Имя группы
        group: String,
        /// Имя профиля цепочки
        #[arg(default_value = "Proxy-Chain")]
        name: String,
        /// Список целевых узлов через запятую
        nodes: String,
    },

    /// Определение формата и парсинг ссылки или файла конфигурации
    #[command(name = "parse")]
    Parse {
        /// URI ссылка или путь к файлу
        input: String,
    },

    /// Запуск плагинов проверки задержек для внешнего файла/ссылки
    #[command(name = "test")]
    Test {
        /// URI ссылка или файл
        input: String,
    },

    /// Генерация Ultimate JSON-конфигурации sing-box
    #[command(name = "buildconfig", alias = "build-config")]
    BuildConfig {
        /// URI ссылка, файл или имя профиля
        input: String,
        /// Путь для сохранения итогового файла
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Включить сетевой интерфейс TUN
        #[arg(long, default_value_t = false)]
        tun: bool,
    },

    /// Создание резервной копии конфигураций, правил и настроек
    #[command(name = "backup")]
    Backup {
        /// Имя резервной копии
        name: String,
        /// Путь для сохранения (по умолчанию .\backups)
        #[arg(default_value = ".\\backups")]
        destination: PathBuf,
        /// Включить конфигурации (true/false)
        #[arg(default_value = "true")]
        configs: String,
        /// Включить правила маршрутизации (true/false)
        #[arg(default_value = "true")]
        routes: String,
        /// Включить настройки приложения (true/false)
        #[arg(default_value = "true")]
        settings: String,
    },

    /// Восстановление конфигураций, правил и настроек из резервной копии
    #[command(name = "recovery", alias = "restore")]
    Recovery {
        /// Путь к файлу резервной копии
        path: PathBuf,
        /// Применить конфигурации (true/false)
        #[arg(default_value = "true")]
        configs: String,
        /// Применить правила маршрутизации (true/false)
        #[arg(default_value = "true")]
        routes: String,
        /// Применить настройки приложения (true/false)
        #[arg(default_value = "true")]
        settings: String,
        /// Режим восстановления: merge (слияние без удаления существующих данных) или hard (полная замена)
        #[arg(short, long, default_value = "merge")]
        mode: String,
        /// Пропустить интерактивное подтверждение (применить автоматически)
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Управление встроенными ассетами (WARP AWG+MASQUE, Goida Group)
    #[command(name = "assets")]
    Assets {
        /// Действие: install (установить в хранилище), list (показать доступные)
        #[arg(default_value = "install")]
        action: String,
    },

    /// Список сохраненных гео-файлов (GeoIP, GeoSite, rule-set) в локальном хранилище
    #[command(name = "geolist")]
    GeoList,

    /// Загрузка и обновление гео-файлов (geoip.db, geosite.db) по сети
    #[command(name = "geoupdate")]
    GeoUpdate {
        /// Имя гео-файла для обновления ("geoip", "geosite" или пусто для всех)
        name: Option<String>,
    },

    /// Загрузка гео-файла из URL или локального файла в LocalStorage
    #[command(name = "geofetch")]
    GeoFetch {
        /// URL или путь к исходному файлу
        source: String,
        /// Желаемое имя в хранилище (например custom-geoip.db)
        name: Option<String>,
        /// Тип гео-файла: geoip, geosite, rule-set
        #[arg(long, default_value = "geoip")]
        geo_type: String,
    },

    /// Удаление гео-файла из LocalStorage
    #[command(name = "geodel")]
    GeoDel {
        /// Имя удаляемого файла
        name: String,
    },
}

fn normalize_cli_args(args: Vec<String>) -> Vec<String> {
    let known_commands = [
        "help", "newconfig", "delconfig", "editconfig", "list", "select",
        "pick", "unpick", "newgroup", "delgroup", "editgroup", "run",
        "ping", "newrule", "editrule", "delrule", "ruleslist", "moverule", "switchrule",
        "settings", "logs", "stop", "new", "edit", "delete", "subinfo",
        "update", "newbalancer", "newchain", "parse", "test", "buildconfig",
        "build-config", "deletegroup", "rules", "backup", "recovery", "restore",
        "geolist", "geoupdate", "geofetch", "geodel", "assets",
    ];

    let mut result = Vec::with_capacity(args.len());
    for (i, arg) in args.into_iter().enumerate() {
        if i == 1 && arg.starts_with('-') && !arg.starts_with("--") {
            let stripped = arg.trim_start_matches('-').to_lowercase();
            if known_commands.contains(&stripped.as_str()) {
                result.push(stripped);
                continue;
            }
        }
        if i > 1 && arg.starts_with('-') && !arg.starts_with("--") {
            let flag = arg.trim_start_matches('-');
            if flag.len() > 1 && flag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                result.push(format!("--{}", flag));
                continue;
            }
        }
        result.push(arg);
    }
    result
}

fn parse_autoupdate_minutes(s: &str) -> u32 {
    let lower = s.trim().to_lowercase();
    if lower.ends_with('h') {
        lower.trim_end_matches('h').trim().parse::<u32>().unwrap_or(1) * 60
    } else if lower.ends_with('d') {
        lower.trim_end_matches('d').trim().parse::<u32>().unwrap_or(1) * 1440
    } else if lower.ends_with('m') {
        lower.trim_end_matches('m').trim().parse::<u32>().unwrap_or(30)
    } else {
        lower.parse::<u32>().unwrap_or(60)
    }
}

fn parse_bool_flag(s: &str, default: bool) -> bool {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "y" | "on" => true,
        "false" | "0" | "no" | "n" | "off" => false,
        _ => default,
    }
}

/// Диагностика доступности Clash API и веб-панели metacubexd.
///
/// Проверяет:
/// 1. Доступен ли endpoint /version (Clash API активен)
/// 2. Доступен ли endpoint /ui (metacubexd загружен)
///
/// Выводит подсказки с возможными решениями при ошибках.
async fn diagnose_clash_api() {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    // Проверка Clash API /version
    match client.get("http://127.0.0.1:9090/version").send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(body) = resp.text().await {
                if let Ok(ver) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(version) = ver.get("version").and_then(|v| v.as_str()) {
                        println!("    - Clash API: \x1b[32mактивен\x1b[0m (sing-box {})", version);
                    }
                }
            }
        }
        Ok(resp) => {
            eprintln!("\x1b[1;33m[ДИАГНОСТИКА] Clash API ответил с кодом {}\x1b[0m", resp.status());
            if resp.status().as_u16() == 401 {
                eprintln!("  Подсказка: проверьте параметр 'secret' в конфигурации clash_api.");
            }
        }
        Err(_) => {
            eprintln!("\x1b[1;33m[ДИАГНОСТИКА] Clash API (127.0.0.1:9090) недоступен.\x1b[0m");
            eprintln!("  Возможные причины:");
            eprintln!("  - Ядро sing-box скомпилировано без тега 'with_clash_api'");
            eprintln!("  - Порт 9090 занят другим процессом");
            eprintln!("  - В конфигурации отсутствует секция experimental.clash_api");
        }
    }

    // Проверка metacubexd UI
    match client.get("http://127.0.0.1:9090/ui/").send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("    - Веб-панель metacubexd: \x1b[32mдоступна\x1b[0m (http://127.0.0.1:9090/ui)");
        }
        Ok(resp) if resp.status().as_u16() == 404 => {
            eprintln!("\x1b[1;33m[ДИАГНОСТИКА] Веб-панель metacubexd НЕ загружена (404).\x1b[0m");
            eprintln!("  Возможные причины:");
            eprintln!("  - Папка 'metacubexd' не найдена рядом с ядром sing-box");
            eprintln!("  - Параметр 'external_ui' указывает на несуществующий путь");
            eprintln!("  Решение: убедитесь, что папка metacubexd/ находится рядом с бинарником,");
            eprintln!("  или добавьте 'external_ui_download_url' в конфигурацию для автозагрузки.");
        }
        _ => {
            // Если Clash API уже не отвечает, не дублируем ошибку
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Информирование об архитектуре x86 (32-bit)
    if CoreSupervisor::is_x86_arch() {
        eprintln!("\x1b[1;33m[ИНФО] Запущена сборка для архитектуры x86 (32-bit).\x1b[0m");
        eprintln!("Режим совместимости с 32-битными системами (Windows 7 x86 / legacy).");
        eprintln!("Для запуска прокси-сессии убедитесь, что в каталоге программы размещен совместимый бинарник singbox.exe.\n");
    }

    // Информирование о режиме Windows 7 legacy
    if CoreSupervisor::is_windows_7() {
        eprintln!("\x1b[1;33m[ПРЕДУПРЕЖДЕНИЕ] Обнаружена Windows 7 (legacy mode).\x1b[0m");
        eprintln!("Полноценная поддержка sing-box на Windows 7 ограничена из-за требований Go runtime.");
        eprintln!("Будет использован legacy-бинарник из singbox/Windows7/ (если доступен).\n");
    }

    let raw_args: Vec<String> = std::env::args().collect();
    let normalized = normalize_cli_args(raw_args);
    let cli = Cli::parse_from(normalized);
    let mut store = ConfigStore::load_or_default(None);

    match cli.command {
        Commands::Help { command_name } => {
            print_custom_help(command_name.as_deref());
        }

        Commands::NewConfig {
            name,
            protocol,
            config_type,
            instant,
            group,
        } => {
            let mut grp_name = group.clone();
            let mut prof_name_override = None;

            if let Some(ref t) = name {
                if t.contains('.') {
                    let mut parts = t.splitn(2, '.');
                    grp_name = parts.next().unwrap().to_string();
                    prof_name_override = parts.next().map(|s| s.to_string());
                } else {
                    prof_name_override = Some(t.clone());
                }
            }

            if store.get_group(&grp_name).is_none() {
                store.add_group(StoredGroup::new_manual(&grp_name))?;
                println!("==> Создана новая группа '{}'.", grp_name);
            }

            let profile = if let Some(raw_input) = instant {
                let json_content = resolve_input(&raw_input)?;
                let mut p = build_profile_instant(&json_content)?;
                if let Some(ov) = prof_name_override {
                    p.name = ov.clone();
                    p.tag = ov;
                }
                p
            } else if config_type.eq_ignore_ascii_case("dialog") {
                let stdin = io::stdin();
                let mut reader = stdin.lock();
                let mut stdout = io::stdout();

                let mut cand_nodes = Vec::new();
                let mut idx = 1;
                for g in &store.data.groups {
                    for p in &g.profiles {
                        cand_nodes.push(core_manager::constructor::CandidateNode {
                            index: idx,
                            group: g.name.clone(),
                            name: p.name.clone(),
                            protocol: p.protocol.to_string(),
                            latency_ms: p.last_ping_ms,
                        });
                        idx += 1;
                    }
                }
                let groups: Vec<String> = store.data.groups.iter().map(|g| g.name.clone()).collect();
                let p_override = protocol.as_deref().map(ProtocolType::from_str);

                core_manager::constructor::config::build_profile_interactive_full(
                    &mut reader,
                    &mut stdout,
                    None,
                    prof_name_override.as_deref(),
                    p_override,
                    &groups,
                    &cand_nodes,
                )?
            } else {
                // Создание базового узла по имени и протоколу
                let p_name = prof_name_override.unwrap_or_else(|| "node-1".to_string());
                let proto = protocol
                    .map(|pr| ProtocolType::from_str(&pr))
                    .unwrap_or(ProtocolType::Vless);

                let default_settings = match proto {
                    ProtocolType::Byedpi => serde_json::json!({ "type": "byedpi", "split_pos": 2 }),
                    ProtocolType::MasterDnsVPN => serde_json::json!({ "type": "masterdnsvpn", "dns_server": "77.88.8.8" }),
                    ProtocolType::Chain => serde_json::json!({ "type": "chain", "nodes": [] }),
                    ProtocolType::Balancer => serde_json::json!({ "type": "balancer", "strategy": "urltest" }),
                    _ => serde_json::Value::Object(serde_json::Map::new()),
                };
                let default_port = match proto {
                    ProtocolType::Byedpi => 1080,
                    ProtocolType::MasterDnsVPN => 53,
                    _ => 443,
                };

                StoredProfile {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: p_name.clone(),
                    protocol: proto,
                    server: "127.0.0.1".to_string(),
                    server_port: default_port,
                    settings: default_settings,
                    tag: p_name,
                    last_ping_ms: None,
                }
            };

            let pname = profile.name.clone();
            store.add_profile(&grp_name, profile)?;
            println!("==> Профиль '{}.{}' успешно сохранен в хранилище.", grp_name, pname);
        }

        Commands::DelConfig { target } => {
            let found = store.find_profile_by_index_or_target(&target);
            if let Some((grp, prof)) = found {
                let full_tag = format!("{}.{}", grp.name, prof.name);
                store.remove_profile(&full_tag)?;
                println!("==> Профиль '{}' успешно удален.", full_tag);
            } else {
                eprintln!("Ошибка: профиль '{}' не найден.", target);
            }
        }

        Commands::EditConfig {
            target,
            config_type,
            instant,
        } => {
            let (grp_name, prof_name, existing_profile) = {
                let (g, p) = store
                    .find_profile_by_index_or_target(&target)
                    .ok_or_else(|| format!("Профиль '{}' не найден", target))?;
                (g.name.clone(), p.name.clone(), p.clone())
            };

            let updated_profile = if let Some(raw_input) = instant {
                let json_content = resolve_input(&raw_input)?;
                let mut p = build_profile_instant(&json_content)?;
                p.id = existing_profile.id;
                p.last_ping_ms = existing_profile.last_ping_ms;
                p
            } else if config_type.eq_ignore_ascii_case("dialog") {
                let stdin = io::stdin();
                let mut reader = stdin.lock();
                let mut stdout = io::stdout();

                let mut cand_nodes = Vec::new();
                let mut idx = 1;
                for g in &store.data.groups {
                    for p in &g.profiles {
                        cand_nodes.push(core_manager::constructor::CandidateNode {
                            index: idx,
                            group: g.name.clone(),
                            name: p.name.clone(),
                            protocol: p.protocol.to_string(),
                            latency_ms: p.last_ping_ms,
                        });
                        idx += 1;
                    }
                }
                let groups: Vec<String> = store.data.groups.iter().map(|g| g.name.clone()).collect();

                core_manager::constructor::config::build_profile_interactive_full(
                    &mut reader,
                    &mut stdout,
                    Some(&existing_profile),
                    Some(&existing_profile.name),
                    Some(existing_profile.protocol.clone()),
                    &groups,
                    &cand_nodes,
                )?
            } else {
                existing_profile
            };

            store.add_profile(&grp_name, updated_profile)?;
            println!("==> Профиль '{}.{}' успешно обновлен.", grp_name, prof_name);
        }

        Commands::List { group } => {
            let mut items = Vec::new();
            let mut idx = 1;

            let target_groups: Vec<&StoredGroup> = if let Some(ref gname) = group {
                let clean_name = gname.trim_start_matches('-').trim();
                store
                    .data
                    .groups
                    .iter()
                    .filter(|g| g.name.eq_ignore_ascii_case(clean_name))
                    .collect()
            } else {
                store.data.groups.iter().collect()
            };

            if target_groups.is_empty() {
                println!("==> Группы не найдены.");
                return Ok(());
            }

            for grp in target_groups {
                for prof in &grp.profiles {
                    let full_tag = format!("{}.{}", grp.name, prof.name);
                    let is_picked = store.data.active_picked.as_deref() == Some(&full_tag);
                    items.push(core_manager::tui::pager::PagerItem {
                        index: idx,
                        group: grp.name.clone(),
                        name: prof.name.clone(),
                        protocol: prof.protocol.to_string(),
                        latency_ms: prof.last_ping_ms,
                        is_picked,
                    });
                    idx += 1;
                }
            }

            let title = if let Some(ref g) = group {
                format!("Список узлов группы '{}'", g.trim_start_matches('-'))
            } else {
                "Все профили NekoBoxForPC".to_string()
            };

            core_manager::tui::pager::display_paged_list(&items, &title)?;
        }

        Commands::Select { selector } => {
            let parsed = ParsedSelector::parse(&selector);
            let selected = parsed.select(&store);
            println!("============================================================");
            println!(" Выбрано узлов по селектору '{}': {}", selector, selected.len());
            println!("============================================================");
            for (i, (grp, prof)) in selected.iter().enumerate() {
                let ping = prof
                    .last_ping_ms
                    .map(|m| format!("{} ms", m))
                    .unwrap_or_else(|| "---".to_string());
                println!(" {:>3}. {}.{:<30} ({}) -> {}", i + 1, grp, prof.name, prof.protocol, ping);
            }
            println!("============================================================");
        }

        Commands::Pick { target } => {
            let chosen_target = if let Some(t) = target {
                if let Some((grp, prof)) = store.find_profile_by_index_or_target(&t) {
                    format!("{}.{}", grp.name, prof.name)
                } else {
                    t
                }
            } else {
                if let Some(t) = interactive_pick_profile(&store)? {
                    t
                } else {
                    return Ok(());
                }
            };

            store.pick(&chosen_target)?;
            println!("==> Узел '{}' успешно выбран активным (PICKED).", chosen_target);
        }

        Commands::Unpick => {
            store.unpick()?;
            println!("==> Активный выбор узла сброшен.");
        }

        Commands::NewGroup {
            name,
            group_type,
            sub,
            autoupdate,
            instant,
        } => {
            let default_name = name.unwrap_or_else(|| "Group".to_string());
            let is_sub = group_type.eq_ignore_ascii_case("sub") || sub.is_some();

            let group = if is_sub {
                let url = sub.unwrap_or_default();
                let mut grp = StoredGroup::new_subscription(&default_name, url);
                if let Some(ref au) = autoupdate {
                    grp.auto_update_minutes = parse_autoupdate_minutes(au);
                }
                grp
            } else if let Some(raw_input) = instant {
                let json_content = resolve_input(&raw_input)?;
                build_group_instant(&json_content)?
            } else {
                StoredGroup::new_manual(&default_name)
            };

            let gname = group.name.clone();
            let is_sub_type = group.group_type == "sub";
            store.add_group(group)?;
            println!("==> Группа '{}' успешно создана.", gname);

            if is_sub_type && !store.get_group(&gname).unwrap().subscription_url.as_deref().unwrap_or("").is_empty() {
                println!("==> Загрузка конфигураций узлов из подписки...");
                if let Ok(res) = SubscriptionHandler::update_group(&mut store, &gname).await {
                    println!("✓ Успешно импортировано {} профилей в группу '{}'.", res.total_count, gname);
                }
            }
        }

        Commands::DelGroup { name } => {
            store.remove_group(&name)?;
            println!("==> Группа '{}' успешно удалена.", name);
        }

        Commands::EditGroup {
            name,
            config_type,
            instant,
        } => {
            let existing = store
                .get_group(&name)
                .ok_or_else(|| format!("Группа '{}' не найдена", name))?
                .clone();

            let updated = if let Some(raw_input) = instant {
                let json_content = resolve_input(&raw_input)?;
                let mut g = build_group_instant(&json_content)?;
                g.profiles = existing.profiles;
                g
            } else if config_type.eq_ignore_ascii_case("dialog") {
                let stdin = io::stdin();
                let mut reader = stdin.lock();
                let mut stdout = io::stdout();
                build_group_interactive(&mut reader, &mut stdout, Some(&existing))?
            } else {
                existing
            };

            store.remove_group(&name)?;
            let new_name = updated.name.clone();
            store.add_group(updated)?;
            println!("==> Группа '{}' успешно обновлена.", new_name);
        }

        Commands::Run {
            target,
            direct,
            chain,
            balancer,
            chain_balancer,
            core,
            tun,
            log_level,
            no_log,
        } => {
            let mut builder = UltimateConfigBuilder::new();

            if direct {
                println!("==> Подготовка сессии: Прямое подключение (Direct Outbound, Mixed порт: 127.0.0.1:20808)");
                builder.inject_direct()?;
            } else if let Some(chain_spec) = chain {
                let mut chain_nodes = Vec::new();
                for item in chain_spec.split(',') {
                    let trim_item = item.trim();
                    if !trim_item.is_empty() {
                        let prof = resolve_profile_target(&mut store, trim_item)?;
                        chain_nodes.push(prof);
                    }
                }
                if chain_nodes.is_empty() {
                    eprintln!("Ошибка: цепочка прокси не содержит валидных узлов.");
                    return Ok(());
                }
                let names: Vec<&str> = chain_nodes.iter().map(|n| n.name.as_str()).collect();
                println!("==> Подготовка сессии: Цепочка из {} узлов: {}", chain_nodes.len(), names.join(" -> "));
                builder.inject_chain(&chain_nodes)?;
            } else if let Some(bal_spec) = balancer {
                let mut candidates = if bal_spec.starts_with("-list") || bal_spec.contains('[') {
                    let clean_sel = bal_spec.replace("-list", "").trim().to_string();
                    let parsed = ParsedSelector::parse(&clean_sel);
                    parsed.select(&store).into_iter().map(|(_, p)| p.into()).collect::<Vec<ProfileConfig>>()
                } else {
                    let grp_name = bal_spec.replace("-group", "").trim().to_string();
                    let grp = store.get_group(&grp_name).ok_or_else(|| format!("Группа '{}' не найдена", grp_name))?;
                    grp.profiles.iter().map(|p| p.clone().into()).collect::<Vec<ProfileConfig>>()
                };

                candidates.retain(|p| p.protocol != ProtocolType::Balancer && p.protocol != ProtocolType::Chain);

                if candidates.is_empty() {
                    eprintln!("Ошибка: для балансировщика '{}' не найдено узлов-кандидатов.", bal_spec);
                    return Ok(());
                }
                println!("==> Подготовка сессии: Балансировщик urltest ({} кандидатов)", candidates.len());
                builder.inject_balancer("proxy", &candidates, None)?;
            } else if let Some(cb_spec) = chain_balancer {
                let mut parts = cb_spec.splitn(2, ',');
                let front_tgt = parts.next().unwrap_or("").trim();
                let group_tgt = parts.next().unwrap_or("").trim();
                let front_node = resolve_profile_target(&mut store, front_tgt)?;
                let grp_name = group_tgt.replace("-group", "").trim().to_string();
                let grp = store.get_group(&grp_name).ok_or_else(|| format!("Группа '{}' не найдена", grp_name))?;
                let mut candidates: Vec<ProfileConfig> = grp.profiles.iter().map(|p| p.clone().into()).collect();
                candidates.retain(|p| p.protocol != ProtocolType::Balancer && p.protocol != ProtocolType::Chain);
                if candidates.is_empty() {
                    eprintln!("Ошибка: группа '{}' пуста.", grp_name);
                    return Ok(());
                }
                println!("==> Подготовка сессии: Цепочка Front [{}] -> Balancer [{}] ({} кандидатов)", front_node.name, grp_name, candidates.len());
                builder.inject_chain_balancer(&front_node, &candidates)?;
            } else {
                let active_profile = match resolve_profile_target_or_picked(&mut store, target.as_deref()) {
                    Ok(p) => p,
                    Err(_) => {
                        if let Some(picked_name) = interactive_pick_profile(&store)? {
                            let _ = store.pick(&picked_name);
                            resolve_profile_target(&mut store, &picked_name)?
                        } else {
                            return Ok(());
                        }
                    }
                };

                if active_profile.protocol == ProtocolType::Balancer {
                    let grp_name = active_profile.settings.get("group").and_then(|v| v.as_str()).unwrap_or("");
                    let candidates = if let Some(grp) = store.get_group(grp_name) {
                        grp.profiles
                            .iter()
                            .filter(|p| p.protocol != ProtocolType::Balancer && p.protocol != ProtocolType::Chain)
                            .map(|p| p.clone().into())
                            .collect::<Vec<ProfileConfig>>()
                    } else {
                        return Err(format!("Группа '{}' для балансировщика не найдена", grp_name).into());
                    };
                    println!("==> Подготовка сессии: Балансировщик urltest [{}] ({} кандидатов из группы '{}')", active_profile.name, candidates.len(), grp_name);
                    builder.inject_balancer("proxy", &candidates, None)?;
                } else if active_profile.protocol == ProtocolType::Chain {
                    let mut chain_nodes = Vec::new();
                    if let Some(nodes) = active_profile.settings.get("nodes").and_then(|v| v.as_array()) {
                        for item in nodes {
                            if let Some(s) = item.as_str() {
                                let prof = resolve_profile_target(&mut store, s)?;
                                chain_nodes.push(prof);
                            }
                        }
                    }
                    if chain_nodes.is_empty() {
                        return Err("Не найдено узлов для цепочки".into());
                    }
                    let names: Vec<&str> = chain_nodes.iter().map(|n| n.name.as_str()).collect();
                    println!("==> Подготовка сессии: Цепочка [{}] ({} узлов: {})", active_profile.name, chain_nodes.len(), names.join(" -> "));
                    builder.inject_chain(&chain_nodes)?;
                } else {
                    println!(
                        "==> Подготовка сессии для: {} | Протокол: {:?} ({}:{})",
                        active_profile.name, active_profile.protocol, active_profile.server, active_profile.server_port
                    );
                    builder.inject_profile(&active_profile)?;
                }
            }

            builder.set_tun_enabled(tun)?;
            builder.set_log_level(&log_level)?;
            builder.inject_rules_and_geo(&store.data.rules, Some(&store.get_geo_dir()))?;

            let config_json = builder.build();

            if tun {
                println!("==> Включен режим системного TUN интерфейса (весь сетевой трафик ОС направлен в прокси).");
            } else {
                println!("==> TUN отключен (работает локальный прокси без системного захвата адаптеров):");
                println!("    - Локальный прокси (Mixed SOCKS5/HTTP): 127.0.0.1:20808");
                println!("    - Панель управления Clash API: http://127.0.0.1:9090/ui");
                println!("    - Быстрый тест в PowerShell: curl.exe -x 127.0.0.1:20808 https://api.ipify.org");
                println!("    - Логи сессии сохраняются в папку: data/logs/ (latest.log)");
                println!("    - Уровень детализации логирования: [{}]", log_level.to_uppercase());
            }

            let supervisor = CoreSupervisor::new(core)?;
            println!("==> Запуск ядра sing-box...");

            let mut log_stream = supervisor.subscribe_logs();
            supervisor.start(config_json).await?;

            if !no_log {
                tokio::spawn(async move {
                    while let Ok(line) = log_stream.recv().await {
                        if (line.contains("ERROR") && !line.contains("NOERROR")) || line.contains("FATAL") || line.contains("panic") {
                            eprintln!("[core ERR]  \x1b[1;31m{}\x1b[0m", line);
                        } else if line.contains("WARN") {
                            println!("[core WARN] \x1b[1;33m{}\x1b[0m", line);
                        } else if line.contains("dns: lookup") || line.contains("dns: exchanged") {
                            println!("[core DNS]  \x1b[35m{}\x1b[0m", line);
                        } else if line.contains("inbound connection from") {
                            println!("[core IN]   \x1b[1;32m{}\x1b[0m", line);
                        } else if line.contains("outbound connection to") {
                            println!("[core OUT]  \x1b[1;36m{}\x1b[0m", line);
                        } else if line.contains("upload finished") || line.contains("download closed") {
                            println!("[core END]  \x1b[90m{}\x1b[0m", line);
                        } else if line.contains("DEBUG") || line.contains("TRACE") {
                            println!("[core TRACE]\x1b[90m{}\x1b[0m", line);
                        } else {
                            println!("[core]      {}", line);
                        }
                    }
                });
            }

            println!("==> Ожидание готовности сервисов и Clash API (127.0.0.1:9090)...");
            if supervisor.wait_for_ready(Duration::from_secs(5)).await {
                println!("==> Прокси успешно активно! Готово к обработке реальных подключений.");
                println!("    - Веб-панель Clash: http://127.0.0.1:9090/ui");
                println!("    - SOCKS5/HTTP вход: 127.0.0.1:20808");
            } else {
                if !supervisor.is_running().await {
                    eprintln!("\n\x1b[1;31m[ОШИБКА] Процесс ядра sing-box аварийно завершился сразу после старта!\x1b[0m");
                    eprintln!("Возможные причины: занят порт 20808 или 9090, либо ошибка параметров конфигурации.");
                    let logs = supervisor.get_log_snapshot().await;
                    if !logs.is_empty() {
                        eprintln!("\x1b[1;33mПоследние строки журнала ядра:\x1b[0m");
                        for l in logs.iter().rev().take(10).rev() {
                            eprintln!("  {}", l);
                        }
                    }
                    return Err("Ядро sing-box не смогло запуститься".into());
                } else {
                    println!("==> Ядро запущено (ожидание подключений клиентов на 127.0.0.1:20808).");
                }
            }

            // Диагностика доступности Clash API и metacubexd
            diagnose_clash_api().await;

            println!("==> Нажмите Ctrl+C для завершения работы...");
            tokio::signal::ctrl_c().await?;

            println!("\n==> Остановка ядра и освобождение ресурсов...");
            supervisor.stop().await?;
            println!("==> Сессия завершена.");
        }

        Commands::Ping {
            selector,
            threads,
            mode,
            sort,
            save,
            no_save,
            export,
        } => {
            let targets = if selector.trim().is_empty() {
                let mut all = Vec::new();
                for g in &store.data.groups {
                    for p in &g.profiles {
                        all.push((g.name.clone(), p.clone()));
                    }
                }
                all
            } else if let Some((grp, prof)) = store.find_profile_by_index_or_target(&selector) {
                vec![(grp.name.clone(), prof.clone())]
            } else if store.get_group(&selector).is_some() {
                let grp = store.get_group(&selector).unwrap();
                grp.profiles.iter().map(|p| (grp.name.clone(), p.clone())).collect()
            } else {
                let parsed = ParsedSelector::parse(&selector);
                parsed.select(&store)
            };

            if targets.is_empty() {
                println!("==> Под условия селектора '{}' не найдено профилей.", selector);
                return Ok(());
            }

            let total = targets.len();
            let default_threads = (num_cpus::get() / 2).max(2).min(30);
            let concurrency = threads.unwrap_or(default_threads);

            println!(
                "==> Запуск параллельного тестирования задержек для {} узлов (параллелизм: {} потоков, режим: {}, сортировка: {})...",
                total, concurrency, mode, sort
            );

            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
            let mut join_set = tokio::task::JoinSet::new();
            let settings = Arc::new(store.data.settings.clone());

            for (idx, (grp_name, p)) in targets.into_iter().enumerate() {
                let sem = semaphore.clone();
                let settings = settings.clone();
                join_set.spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    let reg = PluginRegistry::standard();
                    let profile_cfg: ProfileConfig = p.clone().into();
                    let report = reg.run_all(&profile_cfg, &*settings).await;

                    let mut found_ms = None;
                    for (_plugin_name, r) in report.results {
                        match r {
                            Ok(CheckResult::LatencyMs(ms)) => {
                                found_ms = Some(ms);
                                break;
                            }
                            Ok(CheckResult::HttpCheck { latency_ms, .. }) => {
                                found_ms = Some(latency_ms);
                                break;
                            }
                            _ => {}
                        }
                    }
                    (idx, grp_name, p, found_ms)
                });
            }

            let mut results = Vec::new();
            let mut done_count = 0;

            while let Some(res) = join_set.join_next().await {
                if let Ok(entry) = res {
                    done_count += 1;
                    print!("\r==> Прогресс: [{:>3}/{:>3}] Проверено... ", done_count, total);
                    let _ = io::stdout().flush();
                    results.push(entry);
                }
            }
            println!();

            // Сортировка результатов
            let sort_mode = sort.trim().to_lowercase();
            match sort_mode.as_str() {
                "slowest" | "desc" => {
                    results.sort_by(|a, b| match (a.3, b.3) {
                        (Some(m1), Some(m2)) => m2.cmp(&m1),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.2.name.cmp(&b.2.name),
                    });
                }
                "name" | "alpha" => {
                    results.sort_by(|a, b| a.2.name.to_lowercase().cmp(&b.2.name.to_lowercase()));
                }
                "group" => {
                    results.sort_by(|a, b| {
                        let g_cmp = a.1.to_lowercase().cmp(&b.1.to_lowercase());
                        if g_cmp == std::cmp::Ordering::Equal {
                            a.2.name.to_lowercase().cmp(&b.2.name.to_lowercase())
                        } else {
                            g_cmp
                        }
                    });
                }
                "original" => {
                    results.sort_by_key(|r| r.0);
                }
                _ => {
                    // По умолчанию "fastest" / "asc"
                    results.sort_by(|a, b| match (a.3, b.3) {
                        (Some(m1), Some(m2)) => m1.cmp(&m2),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.2.name.cmp(&b.2.name),
                    });
                }
            }

            let mut success_count = 0;
            let mut sum_ms = 0;
            for (_, _, _, ms) in &results {
                if let Some(m) = ms {
                    success_count += 1;
                    sum_ms += m;
                }
            }
            let avg_ms = if success_count > 0 {
                sum_ms / success_count as u64
            } else {
                0
            };

            // Вывод в терминал в соответствии с mode
            let display_mode = mode.trim().to_lowercase();
            match display_mode.as_str() {
                "pager" => {
                    let pager_items: Vec<core_manager::tui::pager::PagerItem> = results
                        .iter()
                        .enumerate()
                        .map(|(idx, (_, grp_name, p, ms))| {
                            let full_tag = format!("{}.{}", grp_name, p.name);
                            let is_picked = store.data.active_picked.as_deref() == Some(&full_tag);
                            core_manager::tui::pager::PagerItem {
                                index: idx + 1,
                                group: grp_name.clone(),
                                name: p.name.clone(),
                                protocol: p.protocol.to_string(),
                                latency_ms: *ms,
                                is_picked,
                            }
                        })
                        .collect();

                    let title = format!(
                        "Результаты замера задержек (Узлов: {}, Сортировка: {})",
                        total, sort
                    );
                    core_manager::tui::pager::display_paged_list(&pager_items, &title)?;
                }
                "list" => {
                    println!("=========================================================================================");
                    println!(" Результаты замера задержек (Режим: list, Сортировка: {}, Всего: {})", sort, total);
                    println!("=========================================================================================");
                    for (idx, (_, grp_name, p, found_ms)) in results.iter().enumerate() {
                        let full_target = format!("{}.{}", grp_name, p.name);
                        let is_picked = store.data.active_picked.as_deref() == Some(&full_target);
                        let picked_mark = if is_picked { "* [PICKED] " } else { "  " };

                        let (ping_str, color_code) = match found_ms {
                            Some(ms) => {
                                let c = if *ms < 100 {
                                    "\x1b[1;32m"
                                } else if *ms < 300 {
                                    "\x1b[1;33m"
                                } else if *ms < 800 {
                                    "\x1b[35m"
                                } else {
                                    "\x1b[31m"
                                };
                                (format!("{:>4} ms", ms), c)
                            }
                            None => ("timeout".to_string(), "\x1b[1;31m"),
                        };

                        println!(
                            "{:>3}.({}) {}{:<30} | {:<10} | {}{}\x1b[0m",
                            idx + 1,
                            grp_name,
                            picked_mark,
                            p.name,
                            p.protocol.to_string(),
                            color_code,
                            ping_str
                        );
                    }
                    println!("=========================================================================================");
                }
                _ => {
                    // "table" (по умолчанию)
                    println!("=========================================================================================");
                    println!("  #   Задержка   Группа           Имя узла                         Сервер : Порт");
                    println!("=========================================================================================");

                    for (idx, (_, grp_name, p, found_ms)) in results.iter().enumerate() {
                        match found_ms {
                            Some(ms) => {
                                let color_code = if *ms < 100 {
                                    "\x1b[1;32m"
                                } else if *ms < 300 {
                                    "\x1b[1;33m"
                                } else if *ms < 800 {
                                    "\x1b[35m"
                                } else {
                                    "\x1b[31m"
                                };

                                let kind_tag = if p.protocol == ProtocolType::Balancer {
                                    " [БАЛАНСИРОВЩИК]"
                                } else if p.protocol == ProtocolType::Chain {
                                    " [ЦЕПОЧКА]"
                                } else {
                                    ""
                                };

                                println!(
                                    " {:>3}. {}{:4} ms\x1b[0m | {:<14} | {:<30}{} | {}:{}",
                                    idx + 1,
                                    color_code,
                                    ms,
                                    grp_name,
                                    p.name,
                                    kind_tag,
                                    p.server,
                                    p.server_port
                                );
                            }
                            None => {
                                println!(
                                    " {:>3}. \x1b[1;31m timeout\x1b[0m | {:<14} | {:<30} | {}:{}",
                                    idx + 1,
                                    grp_name,
                                    p.name,
                                    p.server,
                                    p.server_port
                                );
                            }
                        }
                    }
                    println!("=========================================================================================");
                }
            }

            println!(
                "==> Тестирование завершено: Успешно: {} / {}, Таймаут: {}, Средний пинг: {} ms",
                success_count,
                total,
                total - success_count,
                avg_ms
            );

            // Сохранение в LocalStorage (store.json)
            let should_save = save && !no_save;
            if should_save {
                let updates: Vec<(String, String, Option<u64>)> = results
                    .iter()
                    .map(|(_, g, p, ms)| (g.clone(), p.name.clone(), *ms))
                    .collect();
                match store.update_pings_batch(&updates) {
                    Ok(count) => {
                        println!("==> Результаты замера ({}) сохранены в LocalStorage (store.json).", count);
                    }
                    Err(e) => {
                        eprintln!("==> Предупреждение: Не удалось сохранить результаты замера: {}", e);
                    }
                }
            } else {
                println!("==> Сохранение результатов в LocalStorage отключено (флаг --no-save).");
            }

            // Экспорт в файл
            if let Some(ref export_path) = export {
                #[derive(serde::Serialize)]
                struct PingExportReport {
                    timestamp: u64,
                    selector: String,
                    mode: String,
                    sort: String,
                    total: usize,
                    success_count: usize,
                    timeout_count: usize,
                    average_ms: u64,
                    results: Vec<PingExportItem>,
                }

                #[derive(serde::Serialize)]
                struct PingExportItem {
                    group: String,
                    name: String,
                    protocol: String,
                    server: String,
                    port: u16,
                    latency_ms: Option<u64>,
                    is_picked: bool,
                }

                let now_ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let export_items: Vec<PingExportItem> = results
                    .iter()
                    .map(|(_, g, p, ms)| {
                        let full_tag = format!("{}.{}", g, p.name);
                        let is_picked = store.data.active_picked.as_deref() == Some(&full_tag);
                        PingExportItem {
                            group: g.clone(),
                            name: p.name.clone(),
                            protocol: p.protocol.to_string(),
                            server: p.server.clone(),
                            port: p.server_port,
                            latency_ms: *ms,
                            is_picked,
                        }
                    })
                    .collect();

                let report = PingExportReport {
                    timestamp: now_ts,
                    selector: selector.clone(),
                    mode: mode.clone(),
                    sort: sort.clone(),
                    total,
                    success_count,
                    timeout_count: total - success_count,
                    average_ms: avg_ms,
                    results: export_items,
                };

                match serde_json::to_string_pretty(&report) {
                    Ok(json_str) => {
                        if let Some(parent) = export_path.parent() {
                            if !parent.as_os_str().is_empty() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                        }
                        if let Err(e) = std::fs::write(export_path, json_str) {
                            eprintln!("==> Ошибка записи файла экспорта '{}': {}", export_path.display(), e);
                        } else {
                            println!("==> Отчет о замере задержек успешно экспортирован в: {}", export_path.display());
                        }
                    }
                    Err(e) => eprintln!("==> Ошибка сериализации отчета экспорта: {}", e),
                }
            }
        }

        Commands::NewRule {
            name,
            action,
            site,
            ip,
            extra_args,
        } => {
            if name.is_none() || action.is_none() {
                // Запуск интерактивного TUI редактора правил
                if let Some(created_rule) = core_manager::tui::rule_editor::run_rule_editor_tui(None)? {
                    store.add_rule(&created_rule.name, &created_rule.action, created_rule.site, created_rule.ip)?;
                    println!("==> Правило '{}' (action: {}) успешно добавлено.", created_rule.name, created_rule.action);
                } else {
                    println!("==> Создание правила отменено.");
                }
            } else {
                let rule_name = name.unwrap();
                let rule_action = action.unwrap();
                let mut sites = Vec::new();
                let mut ips = Vec::new();

                if let Some(s) = site {
                    sites.extend(s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
                }
                if let Some(i) = ip {
                    ips.extend(i.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
                }

                for arg in extra_args {
                    if let Some(rest) = arg.strip_prefix("SITE:") {
                        sites.extend(rest.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
                    } else if let Some(rest) = arg.strip_prefix("IP:") {
                        ips.extend(rest.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
                    }
                }

                store.add_rule(&rule_name, &rule_action, sites, ips)?;
                println!("==> Правило '{}' (action: {}) успешно добавлено.", rule_name, rule_action);
            }
        }

        Commands::EditRule {
            name,
            action,
            site,
            ip,
            extra_args,
        } => {
            let target_name = if let Some(n) = name {
                n
            } else {
                if store.data.rules.is_empty() {
                    println!("==> Список правил маршрутизации пуст.");
                    return Ok(());
                }
                println!("Правила маршрутизации:");
                for (i, r) in store.data.rules.iter().enumerate() {
                    println!("  [{}] {} (action: {}, site: {}, ip: {})", i + 1, r.name, r.action, r.site.len(), r.ip.len());
                }
                print!("Введите имя правила для редактирования: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            };

            let existing_rule = store.data.rules.iter().find(|r| r.name.eq_ignore_ascii_case(&target_name)).cloned()
                .ok_or_else(|| format!("Правило '{}' не найдено", target_name))?;

            // Если не передано параметров модификации через CLI флаги, открываем интерактивный TUI-редактор
            if action.is_none() && site.is_none() && ip.is_none() && extra_args.is_empty() {
                if let Some(updated_rule) = core_manager::tui::rule_editor::run_rule_editor_tui(Some(&existing_rule))? {
                    store.update_rule(&existing_rule.name, updated_rule.clone())?;
                    println!("==> Правило '{}' успешно обновлено.", updated_rule.name);
                } else {
                    println!("==> Редактирование правила отменено.");
                }
            } else {
                let mut updated = existing_rule.clone();
                if let Some(act) = action {
                    updated.action = act;
                }
                if let Some(s) = site {
                    updated.site = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                }
                if let Some(i) = ip {
                    updated.ip = i.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                }
                for arg in extra_args {
                    if let Some(rest) = arg.strip_prefix("SITE:") {
                        updated.site.extend(rest.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
                    } else if let Some(rest) = arg.strip_prefix("IP:") {
                        updated.ip.extend(rest.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
                    }
                }
                store.update_rule(&existing_rule.name, updated)?;
                println!("==> Правило '{}' успешно обновлено.", target_name);
            }
        }

        Commands::DelRule { name } => {
            store.del_rule(&name)?;
            println!("==> Правило '{}' успешно удалено.", name);
        }

        Commands::RulesList => {
            println!("=========================================================================================");
            println!(" #   Статус    Имя правила      Действие   Сайты / Домены           IP / CIDR");
            println!("=========================================================================================");
            if store.data.rules.is_empty() {
                println!("  (список пользовательских правил пуст)");
            } else {
                for (idx, r) in store.data.rules.iter().enumerate() {
                    let status_str = if r.enabled {
                        "\x1b[1;32m[ВКЛ]\x1b[0m "
                    } else {
                        "\x1b[1;31m[ВЫКЛ]\x1b[0m"
                    };
                    let sites_str = if r.site.is_empty() { "-".to_string() } else { r.site.join(", ") };
                    let ips_str = if r.ip.is_empty() { "-".to_string() } else { r.ip.join(", ") };
                    println!(
                        " {:>2}. {} {:<16} {:<10} {:<24} {}",
                        idx + 1, status_str, r.name, r.action, sites_str, ips_str
                    );
                }
            }
            println!("=========================================================================================");
        }

        Commands::MoveRule { name, newpos } => {
            store.move_rule(&name, newpos)?;
            println!("==> Правило '{}' успешно перемещено на позицию {}.", name, newpos);
        }

        Commands::SwitchRule { name, state } => {
            let wanted = state.map(|s| {
                let l = s.to_lowercase();
                l == "on" || l == "true" || l == "1" || l == "enable"
            });
            let new_st = store.switch_rule(&name, wanted)?;
            let st_desc = if new_st { "включено" } else { "отключено" };
            println!("==> Правило '{}' теперь {}.", name, st_desc);
        }

        Commands::Settings => {
            core_manager::tui::settings::run_settings_tui(&mut store)?;
        }

        Commands::Logs {
            lines,
            clear,
            errors,
            name,
        } => {
            let data_logs_dir = core_manager::get_data_dir().join("logs");
            let legacy_logs_dir = PathBuf::from("logs");
            let effective_logs_dir = if data_logs_dir.exists() {
                data_logs_dir
            } else if legacy_logs_dir.exists() {
                legacy_logs_dir
            } else {
                data_logs_dir
            };

            let log_path = if let Some(ref n) = name {
                effective_logs_dir.join(n)
            } else {
                effective_logs_dir.join("latest.log")
            };

            if clear {
                if let Ok(entries) = std::fs::read_dir(&effective_logs_dir) {
                    for entry in entries.flatten() {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
                println!("==> Каталог логов ({}) успешно очищен.", effective_logs_dir.display());
                return Ok(());
            }

            if !log_path.exists() {
                println!("==> Файл логов '{}' пока не существует.", log_path.display());
                return Ok(());
            }

            let content = std::fs::read_to_string(&log_path)?;
            let all_lines: Vec<&str> = if errors {
                content
                    .lines()
                    .filter(|l| {
                        l.contains("ERROR")
                            || l.contains("FATAL")
                            || l.contains("failed")
                            || l.contains("[STDERR]")
                    })
                    .collect()
            } else {
                content.lines().collect()
            };
            let count = all_lines.len();
            let skip = if count > lines { count - lines } else { 0 };

            println!("============================================================");
            if errors {
                println!("        Ошибки в логах sing-box (найдено: {})               ", count);
            } else {
                println!("        Последние {} строк логов: {}                        ", lines.min(count), log_path.display());
            }
            println!("============================================================");
            for l in &all_lines[skip..] {
                if l.contains("ERROR") || l.contains("FATAL") || l.contains("failed") || l.contains("[STDERR]") {
                    eprintln!("\x1b[1;31m{}\x1b[0m", l);
                } else {
                    println!("{}", l);
                }
            }
            println!("============================================================");
        }

        Commands::Stop => {
            println!("==> Остановка активных экземпляров ядра sing-box...");
            #[cfg(target_os = "windows")]
            {
                let output = std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "singbox.exe", "/T"])
                    .output();
                match output {
                    Ok(out) if out.status.success() => {
                        println!("==> Процессы singbox.exe успешно завершены.");
                    }
                    _ => {
                        println!("==> Активных процессов singbox.exe не обнаружено.");
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = std::process::Command::new("pkill")
                    .arg("-f")
                    .arg("singbox")
                    .output();
                println!("==> Процессы singbox завершены.");
            }
        }

        Commands::Update { group } => {
            let target_groups: Vec<String> = if let Some(g) = group {
                vec![g]
            } else {
                store
                    .data
                    .groups
                    .iter()
                    .filter(|g| g.group_type == "sub")
                    .map(|g| g.name.clone())
                    .collect()
            };

            if target_groups.is_empty() {
                println!("==> Нет доступных групп подписок для обновления.");
                return Ok(());
            }

            for gname in target_groups {
                println!("==> Обновление подписки '{}'...", gname);
                match SubscriptionHandler::update_group(&mut store, &gname).await {
                    Ok(res) => {
                        println!("    [Счетчик профилей]: обработано {} узлов", res.total_count);
                        println!("    [Сводка протоколов]: {}", res.format_protocol_stats());
                        println!("✓ Группа '{}' успешно синхронизирована (всего узлов: {}).", gname, res.total_count);
                    }
                    Err(e) => {
                        eprintln!("✗ Ошибка обновления '{}': {}", gname, e);
                    }
                }
            }
            println!("==> Обновление завершено.");
        }

        Commands::SubInfo { group } => {
            let grp = store
                .get_group(&group)
                .ok_or_else(|| format!("Группа '{}' не найдена", group))?;

            println!("============================================================");
            println!("           Информация о подписке: {}", grp.name);
            println!("============================================================");
            println!(" URL: {}", grp.subscription_url.as_deref().unwrap_or("не указан"));
            println!(" Количество узлов в группе: {}", grp.profiles.len());
            if let Some(info) = &grp.subscription_userinfo {
                println!("\n Данные использования трафика:");
                println!("  - Отправлено:   {}", SubscriptionUserInfo::format_bytes(info.upload_bytes));
                println!("  - Скачано:     {}", SubscriptionUserInfo::format_bytes(info.download_bytes));
                println!("  - Всего:       {}", SubscriptionUserInfo::format_bytes(info.used_bytes()));
            }
            println!("============================================================");
        }

        Commands::NewBalancer { group, name } => {
            let grp = store.get_group(&group).ok_or_else(|| format!("Группа '{}' не найдена", group))?;
            let count = grp.profiles.len();
            let balancer_profile = StoredProfile {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.clone(),
                protocol: ProtocolType::Balancer,
                server: "balancer.local".to_string(),
                server_port: 0,
                settings: serde_json::json!({
                    "type": "urltest",
                    "group": group
                }),
                tag: name.clone(),
                last_ping_ms: None,
            };
            store.add_profile(&group, balancer_profile)?;
            println!("==> Балансировщик '{}.{}' (urltest по {} узлам) успешно создан как обычный конфиг!", group, name, count);
        }

        Commands::NewChain { group, name, nodes } => {
            let node_list: Vec<String> = nodes.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            if node_list.is_empty() {
                return Err("Не указаны узлы цепочки".into());
            }
            let chain_profile = StoredProfile {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.clone(),
                protocol: ProtocolType::Chain,
                server: "chain.local".to_string(),
                server_port: 0,
                settings: serde_json::json!({
                    "type": "chain",
                    "nodes": node_list
                }),
                tag: name.clone(),
                last_ping_ms: None,
            };
            store.add_profile(&group, chain_profile)?;
            println!("==> Прокси-цепочка '{}.{}' ({}) успешно сохранена как обычный конфиг!", group, name, nodes);
        }

        Commands::Parse { input } => {
            let content = resolve_input(&input)?;
            let format = detect_config_format(&content);
            println!("==> Формат конфигурации: {:?}", format);
            let res = parse_config(&content);
            if res.is_success() {
                println!("✓ Успешно распознано конфигураций: {}", res.profiles.len());
                for p in res.profiles {
                    println!("  - {} ({}) -> {}:{}", p.name, p.protocol, p.server, p.server_port);
                }
            } else if !res.profiles.is_empty() {
                println!("! Распознано конфигураций (с предупреждениями): {}", res.profiles.len());
                for p in res.profiles {
                    println!("  - {} ({}) -> {}:{}", p.name, p.protocol, p.server, p.server_port);
                }
                for e in res.errors {
                    eprintln!("  ⚠ Предупреждение: {}", e);
                }
            } else {
                eprintln!("✗ Ошибка парсинга:");
                for e in res.errors {
                    eprintln!("  - {}", e);
                }
            }
        }

        Commands::Test { input } => {
            let content = resolve_input(&input)?;
            let parse_res = parse_config(&content);
            if parse_res.profiles.is_empty() {
                return Err(format!("Не удалось извлечь профили для тестирования: {:?}", parse_res.errors).into());
            }
            let reg = PluginRegistry::standard();
            let settings = store.data.settings.clone();
            for p in &parse_res.profiles {
                println!("==> Тестирование: {} ({}:{})", p.name, p.server, p.server_port);
                let report = reg.run_all(p, &settings).await;
                for (plugin_name, r) in report.results {
                    match r {
                        Ok(cr) => println!("  [{}] {}", plugin_name, cr.summary()),
                        Err(e) => println!("  [{}] ✗ Ошибка: {}", plugin_name, e),
                    }
                }
            }
        }

        Commands::BuildConfig { input, output, tun } => {
            let mut builder = UltimateConfigBuilder::new();
            if let Ok(p) = resolve_profile_target(&mut store, &input) {
                builder.inject_profile(&p)?;
            } else {
                let content = resolve_input(&input)?;
                let parse_res = parse_config(&content);
                if let Some(first) = parse_res.profiles.first() {
                    builder.inject_profile(first)?;
                } else {
                    return Err(format!("Не удалось найти или извлечь профиль для сборки: {:?}", parse_res.errors).into());
                }
            }
            builder.set_tun_enabled(tun)?;
            builder.inject_rules_and_geo(&store.data.rules, Some(&store.get_geo_dir()))?;
            let val = builder.build();
            let json_str = serde_json::to_string_pretty(&val)?;
            if let Some(out_path) = output {
                std::fs::write(&out_path, &json_str)?;
                println!("✓ Конфигурация сохранена в: {}", out_path.display());
            } else {
                println!("{}", json_str);
            }
        }

        Commands::Backup {
            name,
            destination,
            configs,
            routes,
            settings,
        } => {
            let inc_configs = parse_bool_flag(&configs, true);
            let inc_routes = parse_bool_flag(&routes, true);
            let inc_settings = parse_bool_flag(&settings, true);

            let target_file = store.create_backup(&name, &destination, inc_configs, inc_routes, inc_settings)?;
            println!("==> Резервная копия успешно создана:");
            println!("    Файл: {}", target_file.display());
            println!("    - Конфигурации: {}", if inc_configs { "ВКЛ (группы и профили)" } else { "ВЫКЛ" });
            println!("    - Маршруты:     {}", if inc_routes { "ВКЛ (правила routing)" } else { "ВЫКЛ" });
            println!("    - Настройки:    {}", if inc_settings { "ВКЛ (settings)" } else { "ВЫКЛ" });
        }

        Commands::Recovery {
            path,
            configs,
            routes,
            settings,
            mode,
            yes,
        } => {
            let app_configs = parse_bool_flag(&configs, true);
            let app_routes = parse_bool_flag(&routes, true);
            let app_settings = parse_bool_flag(&settings, true);
            let rec_mode = core_manager::storage::RecoveryMode::from_str(&mode);

            let preview = store.inspect_backup(&path, app_configs, app_routes, app_settings, rec_mode)?;

            println!("=========================================================================================");
            println!("                       ПОДТВЕРЖДЕНИЕ ВОССТАНОВЛЕНИЯ (RECOVERY)                           ");
            println!("=========================================================================================");
            println!("  Файл бэкапа:  {}", path.display());
            println!("  Имя архива:   {} (версия: {})", preview.backup_name, preview.backup_version);
            let mode_str = match rec_mode {
                core_manager::storage::RecoveryMode::Merge => "MERGE (слияние без удаления существующих данных)",
                core_manager::storage::RecoveryMode::Hard => "HARD (полная замена выбранных компонентов)",
            };
            println!("  Режим:        {}", mode_str);
            println!("-----------------------------------------------------------------------------------------");

            if app_configs {
                let total_p: usize = preview.groups.iter().map(|g| g.profiles_count).sum();
                println!("  [1] КОНФИГУРАЦИИ: ВКЛЮЧЕНО (Групп в бэкапе: {}, Профилей: {})", preview.groups.len(), total_p);
                for g in &preview.groups {
                    println!("      • Группа \"{}\" (узлов: {}) -> [{}]", g.name, g.profiles_count, g.action_desc);
                }
            } else {
                println!("  [1] КОНФИГУРАЦИИ: ПРОПУЩЕНЫ (флаг configs = false)");
            }

            if app_routes {
                println!("  [2] МАРШРУТЫ:     ВКЛЮЧЕНО (Правил в бэкапе: {})", preview.rules.len());
                for r in &preview.rules {
                    println!("      • Правило \"{}\" [{}] -> [{}]", r.name, r.action, r.action_desc);
                }
            } else {
                println!("  [2] МАРШРУТЫ:     ПРОПУЩЕНЫ (флаг routes = false)");
            }

            if app_settings {
                println!("  [3] НАСТРОЙКИ:    ВКЛЮЧЕНО (Изменений параметров: {})", preview.settings_diffs.len());
                for diff in &preview.settings_diffs {
                    println!("      • {}", diff);
                }
                if preview.geo_files_count > 0 {
                    println!("      • Гео-файлы: {} баз в архиве", preview.geo_files_count);
                }
            } else {
                println!("  [3] НАСТРОЙКИ:    ПРОПУЩЕНЫ (флаг settings = false)");
            }

            println!("=========================================================================================");

            if !yes {
                print!("==> Применить восстановление? [y/N]: ");
                let _ = io::stdout().flush();
                let mut input_str = String::new();
                io::stdin().read_line(&mut input_str)?;
                let trimmed = input_str.trim().to_lowercase();
                if trimmed != "y" && trimmed != "yes" && trimmed != "д" && trimmed != "да" {
                    println!("==> Операция восстановления отменена пользователем.");
                    return Ok(());
                }
            }

            println!("==> Выполняется восстановление из бэкапа...");
            let rep = store.restore_backup(&path, app_configs, app_routes, app_settings, rec_mode)?;
            if rep.configs_restored {
                println!("    ✓ Конфигурации восстановлены: групп {}, узлов {}", rep.groups_count, rep.profiles_count);
            }
            if rep.routes_restored {
                println!("    ✓ Маршруты восстановлены: правил {}", rep.rules_count);
            }
            if rep.settings_restored {
                println!("    ✓ Настройки восстановлены");
            }
            println!("==> Хранилище успешно обновлено и сохранено (режим: {:?}).", rec_mode);
        }

        Commands::Assets { action } => {
            match action.trim().to_lowercase().as_str() {
                "list" => {
                    println!("=========================================================================================");
                    println!("                       ВСТРОЕННЫЕ АССЕТЫ (NekoBoxPlusForPC)                              ");
                    println!("=========================================================================================");
                    println!("  [1] Группа: WARP");
                    println!("      • Профиль: Cloudflare WARP (AWG)    | Протокол: WireGuard / AmneziaWG (Jc, Jmin, H1-H4)");
                    println!("      • Профиль: Cloudflare WARP (MASQUE) | Протокол: Vless / MASQUE (HTTP/3 Connect-UDP)");
                    println!("\n  [2] Группа: Goida Group");
                    println!("      • Тип:      Бесплатная подписка (Free Sub) с автообновлением каждые 12 часов");
                    println!("      • Профиль:  🇷🇺 GOIDA Freedom (Bypass)");
                    println!("      • Профиль:  🌐 GOIDA Cloudflare Fallback");
                    println!("=========================================================================================");
                    println!("Для установки в хранилище выполните: nbpfpc assets install (или nbpfpc -assets)");
                }
                _ => {
                    println!("==> Установка встроенных ассетов (WARP + Goida Group)...");
                    let res = store.install_builtin_assets()?;
                    for item in res {
                        println!("    ✓ {}", item);
                    }
                    println!("==> Ассеты успешно добавлены в LocalStorage (store.json).");
                    println!("    Вы можете посмотреть их командой: nbpfpc list или протестировать: nbpfpc ping WARP");
                }
            }
        }

        Commands::GeoList => {
            let geos = store.list_geo_files();
            println!("=========================================================================================");
            println!(" #   Имя файла             Тип       Размер (КБ)  Хеш (SHA256)      Источник");
            println!("=========================================================================================");
            if geos.is_empty() {
                println!("  (в локальном хранилище нет сохраненных гео-файлов)");
            } else {
                for (i, g) in geos.iter().enumerate() {
                    let size_kb = g.size_bytes / 1024;
                    let sha_short = g.sha256.as_deref().map(|s| &s[..10]).unwrap_or("-");
                    let src = g.source_url.as_deref().unwrap_or("локальный");
                    println!(" {:>2}. {:<21} {:<9} {:>8} КБ  {:<10}...  {}", i + 1, g.name, g.file_type, size_kb, sha_short, src);
                }
            }
            println!("=========================================================================================");
            println!("Каталог гео-файлов: {}", store.get_geo_dir().display());
        }

        Commands::GeoUpdate { name } => {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .map_err(|e| format!("Ошибка создания HTTP клиента: {}", e))?;

            let mut targets = Vec::new();
            match name.as_deref() {
                Some("geoip") | Some("geoip.db") => {
                    targets.push(("geoip.db", "geoip", "https://github.com/SagerNet/sing-geoip/releases/latest/download/geoip.db"));
                }
                Some("geosite") | Some("geosite.db") => {
                    targets.push(("geosite.db", "geosite", "https://github.com/SagerNet/sing-geosite/releases/latest/download/geosite.db"));
                }
                None | Some("all") => {
                    targets.push(("geoip.db", "geoip", "https://github.com/SagerNet/sing-geoip/releases/latest/download/geoip.db"));
                    targets.push(("geosite.db", "geosite", "https://github.com/SagerNet/sing-geosite/releases/latest/download/geosite.db"));
                }
                Some(other) => {
                    return Err(format!("Неизвестное имя гео-ресурса '{}'. Доступно: geoip, geosite, all", other).into());
                }
            }

            for (fname, ftype, url) in targets {
                println!("==> Скачивание свежего '{}' с {}...", fname, url);
                match client.get(url).header("User-Agent", "NekoBoxPlusForPC").send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.bytes().await {
                                Ok(bytes) => {
                                    let g = store.save_geo_bytes(fname, ftype, Some(url.to_string()), &bytes)?;
                                    println!("    ✓ Сохранено в LocalStorage: {} ({} КБ, sha256: {})",
                                        g.name, g.size_bytes / 1024, g.sha256.as_deref().unwrap_or("-"));
                                }
                                Err(e) => eprintln!("    ✗ Ошибка чтения тела ответа: {}", e),
                            }
                        } else {
                            eprintln!("    ✗ Ошибка HTTP: {}", resp.status());
                        }
                    }
                    Err(e) => {
                        eprintln!("    ⚠ Не удалось загрузить по сети ({}).", e);
                        eprintln!("      Вы можете добавить локальный файл вручную: nbpfpc geofetch <PATH> {}", fname);
                    }
                }
            }
        }

        Commands::GeoFetch { source, name, geo_type } => {
            let bytes = if source.starts_with("http://") || source.starts_with("https://") {
                println!("==> Загрузка гео-файла по URL: {}...", source);
                let client = reqwest::Client::builder().timeout(Duration::from_secs(60)).build()?;
                let resp = client.get(&source).header("User-Agent", "NekoBoxPlusForPC").send().await?;
                resp.bytes().await?.to_vec()
            } else {
                let p = PathBuf::from(&source);
                if !p.exists() {
                    return Err(format!("Файл '{}' не существует", source).into());
                }
                println!("==> Импорт локального гео-файла '{}'...", p.display());
                std::fs::read(&p)?
            };

            let final_name = name.unwrap_or_else(|| {
                PathBuf::from(&source)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("geo_custom.db")
                    .to_string()
            });

            let src_url = if source.starts_with("http") { Some(source) } else { None };
            let g = store.save_geo_bytes(&final_name, &geo_type, src_url, &bytes)?;
            println!("==> Гео-файл '{}' (тип: {}) успешно сохранен в LocalStorage ({} байт).", g.name, g.file_type, g.size_bytes);
        }

        Commands::GeoDel { name } => {
            store.remove_geo_file(&name)?;
            println!("==> Гео-файл '{}' успешно удален из LocalStorage.", name);
        }
    }

    Ok(())
}

fn print_custom_help(command: Option<&str>) {
    match command {
        Some("newconfig") => {
            println!("Справка: nbpfpc newconfig [NAME] [OPTIONS]");
            println!("Создание профиля подключения (обычный или диалоговый).");
            println!("  Параметры:");
            println!("    --protocol <TYPE>     Протокол узла:");
            println!("                          vless, vmess, trojan, shadowsocks (ss), hysteria2, wireguard,");
            println!("                          amneziawg (awg), tuic, ssh, byedpi, masterdnsvpn, proxychain, balancer, direct");
            println!("    --type <usual|dialog> Режим создания: usual (обычный), dialog (интерактивный мастер)");
            println!("    --instant <JSON/URI>  Моментальная строка JSON аутбаунда или vless:// ссылка");
            println!("    --group <GROUP>       Имя целевой группы (по умолчанию 'Default')");
        }
        Some("list") => {
            println!("Справка: nbpfpc list [GROUP]");
            println!("Постраничный интерактивный просмотр узлов с автоопределением высоты экрана.");
            println!("  Навигация:  [← / →] листать ±1 стр, [PgUp / PgDn] ±2 стр, [Home/End] начало/конец");
            println!("  Сортировка: [↑ / ↓] или [W / S] смена режима сортировки (Default, Ping, Name, Group, Protocol)");
            println!("  Поиск:      [/] или [Ctrl+F] быстрый поиск узлов.");
            println!("  Выход:      [Q / Й] или [Esc] завершить просмотр.");
        }
        Some("ping") => {
            println!("Справка: nbpfpc ping [TARGET] [OPTIONS]");
            println!("Параллельное тестирование задержек узлов с сохранением и сортировкой.");
            println!("  TARGET:                  имя группы, узла или диапазон [a-c]. Пусто = все узлы.");
            println!("  -t, --threads <N>:       количество параллельных потоков (по умолчанию доступные ядра / 2)");
            println!("  -m, --mode <MODE>:       режим вывода: table (таблица), list (список как в list), pager (TUI-пейджер)");
            println!("  -s, --sort <CRITERIA>:   сортировка: fastest (asc), slowest (desc), name, group, original");
            println!("  --save / --no-save:      сохранять ли задержки в LocalStorage (store.json, по умолчанию: да)");
            println!("  --export <FILE>:         путь для сохранения отчета о замере в формате JSON");
        }
        Some("run") => {
            println!("Справка: nbpfpc run [TARGET] [OPTIONS]");
            println!("Запуск прокси-сессии sing-box без TUN (SOCKS5/HTTP: 127.0.0.1:20808, Clash API: 9090).");
            println!("  --log-level <trace|debug|info>: уровень логирования (по умолчанию trace).");
            println!("  --no-log: скрыть поток логов.");
        }
        Some("rules") | Some("newrule") | Some("editrule") | Some("delrule") | Some("ruleslist") | Some("moverule") | Some("switchrule") => {
            println!("Справка по правилам маршрутизации:");
            println!("  nbpfpc newrule [NAME] [ACTION] [SITE:...] [IP:...]");
            println!("    (если параметры не указаны — открывается интерактивный TUI-редактор с автодополнениями)");
            println!("  nbpfpc editrule [NAME]");
            println!("    (интерактивное окно редактирования доменов и IP с окном пресетов geosite/geoip/ruleset)");
            println!("  nbpfpc delrule <NAME>");
            println!("  nbpfpc ruleslist");
            println!("  nbpfpc moverule <NAME> <NEWPOS>");
            println!("  nbpfpc switchrule <NAME> [on|off]");
        }
        Some("backup") => {
            println!("Справка: nbpfpc backup <NAME> [DESTINATION] [CONFIGS] [ROUTES] [SETTINGS]");
            println!("Создание резервной копии состояния.");
            println!("  NAME:        Имя файла резервной копии (например 'my_backup')");
            println!("  DESTINATION: Папка сохранения (по умолчанию '.\\backups')");
            println!("  CONFIGS:     Включить конфигурации групп и узлов: true/false (дефолт: true)");
            println!("  ROUTES:      Включить правила маршрутизации: true/false (дефолт: true)");
            println!("  SETTINGS:    Включить настройки: true/false (дефолт: true)");
        }
        Some("recovery") | Some("restore") => {
            println!("Справка: nbpfpc recovery <PATH> [CONFIGS] [ROUTES] [SETTINGS] [OPTIONS]");
            println!("Восстановление данных из резервной копии с окном подтверждения и режимами.");
            println!("  PATH:                    Путь к файлу бэкапа (например '.\\backups\\my_backup.json')");
            println!("  CONFIGS:                 Применить конфигурации: true/false (дефолт: true)");
            println!("  ROUTES:                  Применить правила маршрутизации: true/false (дефолт: true)");
            println!("  SETTINGS:                Применить настройки: true/false (дефолт: true)");
            println!("  -m, --mode <merge|hard>: Режим: merge (слияние без удаления), hard (полная замена)");
            println!("  -y, --yes:               Пропустить окно подтверждения и применить сразу");
        }
        Some("assets") => {
            println!("Справка: nbpfpc assets [install|list]");
            println!("Встроенные готовые шаблоны и ресурсы (WARP AWG+MASQUE, Goida Group Free Sub).");
            println!("  install (дефолт):        Установить группы WARP и Goida Group в LocalStorage");
            println!("  list:                    Показать информацию о доступных встроенных ассетах");
        }
        Some("geo") | Some("geolist") | Some("geoupdate") | Some("geofetch") | Some("geodel") => {
            println!("Справка по гео-файлам (Local Storage):");
            println!("  nbpfpc geolist                       Список сохраненных гео-баз");
            println!("  nbpfpc geoupdate [NAME]              Обновить базы geoip.db / geosite.db по сети");
            println!("  nbpfpc geofetch <URL|PATH> [NAME]    Загрузить гео-файл из URL или диска");
            println!("  nbpfpc geodel <NAME>                 Удалить гео-файл из хранилища");
        }
        _ => {
            println!("============================================================");
            println!("              NekoBoxPlusForPC CLI — Руководство            ");
            println!("============================================================");
            println!("КОНФИГИ И ГРУППЫ:");
            println!("  nbpfpc list [GROUP]                  Постраничный просмотр (пейджер с поиском)");
            println!("  nbpfpc newconfig [NAME]              Создать узел (обычный или dialog)");
            println!("  nbpfpc delconfig <NAME|INDEX>        Удалить узел по имени или номеру");
            println!("  nbpfpc editconfig <NAME|INDEX>       Редактировать узел");
            println!("  nbpfpc pick [TARGET]                 Выбрать активный узел (или smart-меню)");
            println!("  nbpfpc unpick                        Снять выбор");
            println!("  nbpfpc select <SELECTOR>             Выбрать узлы по диапазону [a-c], [1-5] или номерам");
            println!("  nbpfpc newgroup <NAME> [--sub URL]   Создать группу (ручную или подписку)");
            println!("  nbpfpc delgroup <NAME>               Удалить группу");
            println!("  nbpfpc editgroup <NAME>              Редактировать группу");
            println!("  nbpfpc assets [install|list]         Встроенные ассеты (WARP AWG+MASQUE, Goida Group)");
            println!("\nСЕТЬ И ЯДРО:");
            println!("  nbpfpc run [TARGET]                  Запуск сессии ядра (SOCKS5 20808 + Web UI 9090)");
            println!("  nbpfpc ping [TARGET] [OPTIONS]       Тест задержек (--mode table|list|pager, --sort, --export)");
            println!("  nbpfpc stop                          Остановить ядро sing-box");
            println!("  nbpfpc logs [-l N] [-e]              Логи ядра (-e только ошибки)");
            println!("\nМАРШРУТИЗАЦИЯ И ПРАВИЛА:");
            println!("  nbpfpc newrule <NAME> <ACTION> ...   Добавить правило (SITE: IP:)");
            println!("  nbpfpc delrule <NAME>                Удалить правило");
            println!("  nbpfpc ruleslist                     Список всех правил маршрутизации");
            println!("  nbpfpc moverule <NAME> <POS>         Изменить приоритет правила");
            println!("  nbpfpc switchrule <NAME> [on|off]    Включить / выключить правило");
            println!("\nБЭКАП И ВОССТАНОВЛЕНИЕ:");
            println!("  nbpfpc backup <NAME> [DEST] [C] [R] [S]  Создать бэкап (configs, routes, settings)");
            println!("  nbpfpc recovery <PATH> [C] [R] [S] [-m]  Восстановить (merge/hard + окно подтверждения)");
            println!("\nГЕО-ДАННЫЕ (LOCAL STORAGE):");
            println!("  nbpfpc geolist                       Список сохраненных гео-баз");
            println!("  nbpfpc geoupdate [NAME]              Обновить geoip.db / geosite.db по сети");
            println!("  nbpfpc geofetch <URL|PATH> [NAME]    Импортировать гео-файл из URL/диска");
            println!("  nbpfpc geodel <NAME>                 Удалить гео-файл");
            println!("\nНАСТРОЙКИ:");
            println!("  nbpfpc settings                      Интерактивное TUI меню настроек");
            println!("============================================================");
            println!("Поддерживаются вызовы как с дефисом (-list, -ping, -recovery), так и без (list, ping).");
        }
    }
}

fn resolve_input(raw: &str) -> Result<String, String> {
    let p = PathBuf::from(raw);
    if p.exists() {
        std::fs::read_to_string(&p).map_err(|e| format!("Ошибка чтения файла '{}': {}", raw, e))
    } else {
        Ok(raw.to_string())
    }
}

fn resolve_profile_target(
    store: &mut ConfigStore,
    target: &str,
) -> Result<ProfileConfig, String> {
    if let Some((_grp, prof)) = store.find_profile_by_index_or_target(target) {
        return Ok(prof.clone().into());
    }

    let p = PathBuf::from(target);
    if p.exists() {
        let content = std::fs::read_to_string(&p)
            .map_err(|e| format!("Ошибка чтения файла '{}': {}", target, e))?;
        let res = parse_config(&content);
        if let Some(first) = res.profiles.into_iter().next() {
            return Ok(first);
        }
    }

    if target.contains("://") {
        let res = parse_config(target);
        if let Some(first) = res.profiles.into_iter().next() {
            return Ok(first);
        }
    }

    Err(format!("Не удалось найти или распарсить узел '{}'", target))
}

fn resolve_profile_target_or_picked(
    store: &mut ConfigStore,
    target: Option<&str>,
) -> Result<ProfileConfig, String> {
    if let Some(tgt) = target {
        return resolve_profile_target(store, tgt);
    }

    if let Some(picked) = store.data.active_picked.clone() {
        return resolve_profile_target(store, &picked);
    }

    if let Some(last) = store.data.last_run.clone() {
        return resolve_profile_target(store, &last);
    }

    Err("Не указан узел и нет активного выбора (PICKED)".into())
}

fn interactive_pick_profile(store: &ConfigStore) -> io::Result<Option<String>> {
    let groups = &store.data.groups;
    if groups.is_empty() {
        println!("==> Хранилище пусто.");
        return Ok(None);
    }

    println!("============================================================");
    println!("              ВЫБОР ГРУППЫ ПРОФИЛЕЙ                         ");
    println!("============================================================");
    for (idx, g) in groups.iter().enumerate() {
        let gtype = if g.group_type == "sub" { "подписка" } else { "ручная" };
        println!("  [{:>2}] {:<20} (тип: {}, узлов: {})", idx + 1, g.name, gtype, g.profiles.len());
    }
    println!("============================================================");
    print!("Выберите номер группы (или Q / Й для выхода): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("q") || trimmed == "й" || trimmed == "Й" {
        return Ok(None);
    }

    let group_idx = match trimmed.parse::<usize>() {
        Ok(num) if num >= 1 && num <= groups.len() => num - 1,
        _ => {
            if let Some(pos) = groups.iter().position(|g| g.name.eq_ignore_ascii_case(trimmed)) {
                pos
            } else {
                println!("Неверный выбор группы.");
                return Ok(None);
            }
        }
    };

    let chosen_group = &groups[group_idx];
    if chosen_group.profiles.is_empty() {
        println!("В группе '{}' нет доступных профилей.", chosen_group.name);
        return Ok(None);
    }

    println!("============================================================");
    println!("     ВЫБОР УЗЛА В ГРУППЕ: {} (всего: {})", chosen_group.name, chosen_group.profiles.len());
    println!("============================================================");

    for (idx, p) in chosen_group.profiles.iter().enumerate() {
        let ping_str = match p.last_ping_ms {
            Some(ms) => format!("{:>4} ms", ms),
            None => " --- ms".to_string(),
        };
        let is_picked = store.data.active_picked.as_deref() == Some(&format!("{}.{}", chosen_group.name, p.name));
        let mark = if is_picked { "* [PICKED]" } else { " " };
        println!(
            "  [{:>3}] {} {:<32} | {:<10} | {}",
            idx + 1, mark, p.name, p.protocol.to_string(), ping_str
        );
    }
    println!("============================================================");
    print!("Выберите номер узла (или Q / Й для выхода): ");
    io::stdout().flush()?;

    let mut prof_input = String::new();
    io::stdin().read_line(&mut prof_input)?;
    let trimmed_prof = prof_input.trim();
    if trimmed_prof.is_empty() || trimmed_prof.eq_ignore_ascii_case("q") || trimmed_prof == "й" || trimmed_prof == "Й" {
        return Ok(None);
    }

    let prof_idx = match trimmed_prof.parse::<usize>() {
        Ok(num) if num >= 1 && num <= chosen_group.profiles.len() => num - 1,
        _ => {
            if let Some(pos) = chosen_group.profiles.iter().position(|p| p.name.eq_ignore_ascii_case(trimmed_prof)) {
                pos
            } else {
                println!("Неверный выбор профиля.");
                return Ok(None);
            }
        }
    };

    let chosen_prof = &chosen_group.profiles[prof_idx];
    Ok(Some(format!("{}.{}", chosen_group.name, chosen_prof.name)))
}
