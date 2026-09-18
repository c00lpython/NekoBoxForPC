//! Интерактивный терминальный TUI для просмотра и редактирования настроек приложения.
//! Реализует двухуровневую структуру: список категорий -> экран параметров категории.
//! Оснащен фильтрацией KeyEventKind::Press (устранение переключения через один)
//! и поддержкой кросс-раскладки (Q/Й, W/Ц, S/Ы, Enter/Space).

use crate::storage::{AppSettings, ConfigStore};
use crate::tui::keyboard::{is_exit_key, is_key_press, normalize_char};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, stdout, IsTerminal, Write};

struct RawGuard;
impl Drop for RawGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub fn run_settings_tui(store: &mut ConfigStore) -> io::Result<()> {
    if !stdout().is_terminal() {
        print_flat_settings(&store.data.settings);
        return Ok(());
    }

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, Hide)?;
    let _guard = RawGuard;

    let mut selected_cat: usize = 0;
    let mut current_category: Option<usize> = None;
    let mut selected_item: usize = 0;
    let mut modified = false;

    loop {
        let is_en = store.data.settings.language == "en";
        let categories = if is_en {
            vec![
                ("Interface", "language, theme, icons, addresses, country flags"),
                ("Connection", "VPN/proxy mode, network stack (mixed, gvisor, system)"),
                ("Core", "Mux protocol (h2mux, yamux, s2mux), memory limit, SSL certs"),
                ("Income", "local proxy in VPN, UDP, local IP, port, auth, LAN 0.0.0.0, bypass"),
                ("Routing", "apps routing, unknown TUN action, DNS TUN, bypass LAN, rule sources"),
                ("DNS", "DNS routing, FakeDNS, direct DNS, DoH, IPv6, geo resources"),
                ("Tests", "latency URL, geo lookup URL, speedtest, timeout, method"),
                ("Logging", "log verbosity level, log file save path"),
                ("Other", "uTLS fingerprint, min TLS version, autoupdate subs/rules"),
            ]
        } else {
            vec![
                ("Интерфейс", "язык интерфейса, тема, иконки, адреса, флаги стран"),
                ("Подключение", "режим VPN/Proxy, сетевой стек (mixed, gvisor, system)"),
                ("Ядро", "протокол Mux (h2mux, yamux, s2mux), лимит памяти, проверка SSL"),
                ("Входящие", "локальный прокси в VPN, UDP, IP, порт, логин/пароль, LAN 0.0.0.0"),
                ("Маршрутизация", "маршрутизация приложений, правила TUN, DNS TUN, обход LAN"),
                ("DNS", "DNS-маршрутизация, FakeDNS, прямой DNS, DoH, IPv6, гео-базы"),
                ("Тесты", "URL проверки задержки, гео-URL, спидтест, таймаут, метод"),
                ("Логирование", "уровень детализации логов, путь к файлу логов"),
                ("Прочее", "uTLS фингерпринт, версия TLS, автообновление подписок и баз"),
            ]
        };

        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;

        if let Some(cat_idx) = current_category {
            // === УРОВЕНЬ 1: Экран параметров конкретной категории ===
            let cat_name = categories[cat_idx].0;
            let title = if is_en {
                format!("=== NekoBoxForPC: Settings -> {} ===", cat_name)
            } else {
                format!("=== NekoBoxForPC: Настройки -> {} ===", cat_name)
            };

            execute!(
                out,
                SetForegroundColor(Color::Cyan),
                Print("============================================================\r\n"),
                Print(format!("   {}\r\n", title)),
                Print("============================================================\r\n"),
                ResetColor
            )?;

            let items = get_category_items(&store.data.settings, cat_idx, is_en);
            let items_count = items.len();

            if selected_item >= items_count && items_count > 0 {
                selected_item = items_count - 1;
            }

            for (idx, (label, val)) in items.iter().enumerate() {
                let is_sel = idx == selected_item;
                let mark = if is_sel { "[*]" } else { "[ ]" };
                let color = if is_sel { Color::Yellow } else { Color::White };
                let hint = if is_sel {
                    if is_en { " <-- [Enter / Space: toggle]" } else { " <-- [Enter / Пробел: переключить]" }
                } else {
                    ""
                };

                execute!(
                    out,
                    SetForegroundColor(color),
                    Print(format!(" {} {:<32} {}{}\r\n", mark, label, val, hint)),
                    ResetColor
                )?;
            }

            let footer_nav = if is_en {
                "  ↑ / ↓ / W / S Navigate  |  Enter / Space Toggle  |  Q / Esc Back to Menu\r\n"
            } else {
                "  ↑ / ↓ / W / S Навигация  |  Enter / Пробел Изменить  |  Q / Esc Назад в меню\r\n"
            };

            execute!(
                out,
                Print("\r\n"),
                SetForegroundColor(Color::DarkCyan),
                Print("------------------------------------------------------------\r\n"),
                SetForegroundColor(Color::Green),
                Print(footer_nav),
                ResetColor
            )?;

            out.flush()?;

            if let Event::Key(key) = event::read()? {
                if !is_key_press(&key) {
                    continue;
                }
                if is_exit_key(&key) {
                    // Возврат на уровень выше (в главное меню) без закрытия приложения
                    current_category = None;
                    continue;
                }
                match key.code {
                    KeyCode::Up => {
                        if selected_item > 0 {
                            selected_item -= 1;
                        } else if items_count > 0 {
                            selected_item = items_count - 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_item + 1 < items_count {
                            selected_item += 1;
                        } else {
                            selected_item = 0;
                        }
                    }
                    KeyCode::Char(c) => {
                        let n = normalize_char(c);
                        if n == 'w' || n == 'k' {
                            if selected_item > 0 {
                                selected_item -= 1;
                            } else if items_count > 0 {
                                selected_item = items_count - 1;
                            }
                        } else if n == 's' || n == 'j' {
                            if selected_item + 1 < items_count {
                                selected_item += 1;
                            } else {
                                selected_item = 0;
                            }
                        } else if c == ' ' {
                            toggle_setting_item(&mut store.data.settings, cat_idx, selected_item);
                            modified = true;
                        }
                    }
                    KeyCode::Enter => {
                        toggle_setting_item(&mut store.data.settings, cat_idx, selected_item);
                        modified = true;
                    }
                    _ => {}
                }
            }
        } else {
            // === УРОВЕНЬ 0: Главное меню (только список категорий) ===
            let header_title = if is_en {
                "             NekoBoxForPC: SYSTEM SETTINGS                  \r\n"
            } else {
                "             NekoBoxForPC: НАСТРОЙКИ СИСТЕМЫ               \r\n"
            };

            execute!(
                out,
                SetForegroundColor(Color::Cyan),
                Print("============================================================\r\n"),
                Print(header_title),
                Print("============================================================\r\n"),
                ResetColor
            )?;

            for (idx, (cat_name, cat_desc)) in categories.iter().enumerate() {
                let mark = if idx == selected_cat { "[*]" } else { "[ ]" };
                let color = if idx == selected_cat {
                    Color::Yellow
                } else {
                    Color::White
                };

                execute!(
                    out,
                    SetForegroundColor(color),
                    Print(format!(" {} {:<15} - {}\r\n", mark, cat_name, cat_desc)),
                    ResetColor
                )?;
            }

            let footer_nav = if is_en {
                "  ↑ / ↓ / W / S Navigate  |  Enter Open Category  |  Q / Esc Exit\r\n"
            } else {
                "  ↑ / ↓ / W / S Навигация  |  Enter Открыть категорию  |  Q / Esc Выход\r\n"
            };

            execute!(
                out,
                Print("\r\n"),
                SetForegroundColor(Color::DarkCyan),
                Print("------------------------------------------------------------\r\n"),
                SetForegroundColor(Color::Green),
                Print(footer_nav),
                ResetColor
            )?;

            out.flush()?;

            if let Event::Key(key) = event::read()? {
                if !is_key_press(&key) {
                    continue;
                }
                if is_exit_key(&key) {
                    break;
                }
                match key.code {
                    KeyCode::Up => {
                        if selected_cat > 0 {
                            selected_cat -= 1;
                        } else {
                            selected_cat = categories.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_cat + 1 < categories.len() {
                            selected_cat += 1;
                        } else {
                            selected_cat = 0;
                        }
                    }
                    KeyCode::Char(c) => {
                        let n = normalize_char(c);
                        if n == 'w' || n == 'k' {
                            if selected_cat > 0 {
                                selected_cat -= 1;
                            } else {
                                selected_cat = categories.len() - 1;
                            }
                        } else if n == 's' || n == 'j' {
                            if selected_cat + 1 < categories.len() {
                                selected_cat += 1;
                            } else {
                                selected_cat = 0;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Переход на уровень глубже в категорию
                        current_category = Some(selected_cat);
                        selected_item = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    if modified {
        let _ = store.save();
    }

    Ok(())
}

fn get_category_items(s: &AppSettings, cat_idx: usize, is_en: bool) -> Vec<(String, String)> {
    match cat_idx {
        0 => vec![
            (
                if is_en { "Language (Язык):" } else { "Язык интерфейса:" }.to_string(),
                if s.language == "en" { "English (EN)" } else { "Русский (RU)" }.to_string(),
            ),
            (if is_en { "Theme:" } else { "Тема:" }.to_string(), s.theme.clone()),
            (if is_en { "Show icons:" } else { "Показывать иконки:" }.to_string(), s.show_icons.to_string()),
            (if is_en { "Country/flags in profile:" } else { "Флаги стран в профиле:" }.to_string(), s.show_country.to_string()),
            (if is_en { "Display address:" } else { "Отображать адреса:" }.to_string(), s.display_address.to_string()),
        ],
        1 => vec![
            (if is_en { "Connection mode:" } else { "Режим подключения:" }.to_string(), s.connection_mode.clone()),
            (if is_en { "TUN Network stack:" } else { "TUN Сетевой стек:" }.to_string(), s.stack.clone()),
        ],
        2 => vec![
            (if is_en { "Mux protocol:" } else { "Протокол Mux:" }.to_string(), s.mux_protocol.clone()),
            (if is_en { "Memory limit (MB):" } else { "Лимит памяти (МБ):" }.to_string(), format!("{} MB", s.memory_limit_mb)),
            (if is_en { "Cert validation (SSL):" } else { "Проверка сертификатов SSL:" }.to_string(), s.cert_validation.to_string()),
        ],
        3 => vec![
            (if is_en { "Local proxy in VPN:" } else { "Локальный прокси в VPN:" }.to_string(), s.income_local_proxy_in_vpn.to_string()),
            (if is_en { "Turn off UDP for local proxy:" } else { "Отключить UDP для прокси:" }.to_string(), s.income_turn_off_udp.to_string()),
            (if is_en { "Local IP:" } else { "Локальный IP:" }.to_string(), s.income_local_ip.clone()),
            (if is_en { "Local port:" } else { "Локальный порт:" }.to_string(), s.income_local_port.to_string()),
            (if is_en { "Username:" } else { "Имя пользователя:" }.to_string(), if s.income_username.is_empty() { "<none>".to_string() } else { s.income_username.clone() }),
            (if is_en { "Password:" } else { "Пароль:" }.to_string(), if s.income_password.is_empty() { "<none>".to_string() } else { "******".to_string() }),
            (if is_en { "HTTP proxy in VPN:" } else { "HTTP прокси в VPN:" }.to_string(), s.income_http_proxy_in_vpn.to_string()),
            (if is_en { "Bypass:" } else { "Прямой обход (Bypass):" }.to_string(), s.income_bypass.to_string()),
            (if is_en { "Hard route:" } else { "Жесткая маршрутизация:" }.to_string(), s.income_hard_route.to_string()),
            (if is_en { "Access from LAN (0.0.0.0):" } else { "Доступ из LAN (0.0.0.0):" }.to_string(), s.income_access_from_lan.to_string()),
        ],
        4 => vec![
            (if is_en { "Apps routing:" } else { "Маршрутизация приложений:" }.to_string(), s.routing_apps.to_string()),
            (if is_en { "Unknown TUN-traffic ACTION:" } else { "Неизвестный TUN-трафик:" }.to_string(), s.routing_unknown_tun_traffic.clone()),
            (if is_en { "System DNS-traffic TUN:" } else { "Системный DNS-трафик TUN:" }.to_string(), s.routing_system_dns_tun.clone()),
            (if is_en { "DNS TUN (bypass system polytics):" } else { "DNS TUN (обход политик TUN):" }.to_string(), s.routing_dns_tun.to_string()),
            (if is_en { "DoT TUN:" } else { "DoT через TUN:" }.to_string(), s.routing_dot_tun.to_string()),
            (if is_en { "DoH TUN:" } else { "DoH через TUN:" }.to_string(), s.routing_doh_tun.to_string()),
            (if is_en { "Bypass LAN:" } else { "Обход локальной сети (LAN):" }.to_string(), s.routing_bypass_lan.to_string()),
            (if is_en { "Bypass LAN in Core:" } else { "Обход LAN на уровне ядра:" }.to_string(), s.routing_bypass_lan_in_core.to_string()),
            (if is_en { "DPI (for routing only):" } else { "DPI (только для маршрутизации):" }.to_string(), s.routing_dpi.to_string()),
            (if is_en { "Resolve destination (SNI -> IPv6):" } else { "Резолв назначения (SNI->IPv6):" }.to_string(), s.routing_resolve_destination.to_string()),
            (if is_en { "IPv6 Route:" } else { "Маршрутизация IPv6:" }.to_string(), s.routing_ipv6_route.to_string()),
            (if is_en { "Rules source:" } else { "Источник правил (Rules source):" }.to_string(), s.routing_rules_source.clone()),
            (if is_en { "Autoupdate rules period:" } else { "Период обновления правил:" }.to_string(), format!("{} дней", s.routing_autoupdate_period_days)),
        ],
        5 => vec![
            (if is_en { "DNS routing:" } else { "DNS маршрутизация:" }.to_string(), s.dns_routing_enabled.to_string()),
            (if is_en { "FakeDNS:" } else { "FakeDNS:" }.to_string(), s.fake_dns.to_string()),
            (if is_en { "Direct DNS:" } else { "Прямой DNS:" }.to_string(), s.direct_dns.clone()),
            (if is_en { "Proxy DNS (DoH):" } else { "Прокси DNS (DoH):" }.to_string(), s.proxy_dns.clone()),
            (if is_en { "User DNS servers:" } else { "Пользовательские DNS:" }.to_string(), s.user_dns.join(", ")),
            (if is_en { "Enable IPv6:" } else { "Поддержка IPv6:" }.to_string(), s.enable_ipv6.to_string()),
            (if is_en { "Geo resource:" } else { "Гео-ресурс:" }.to_string(), s.geo_resource.clone()),
        ],
        6 => vec![
            (if is_en { "Latency test URL:" } else { "URL проверки задержки:" }.to_string(), s.test_url.clone()),
            (if is_en { "Geo lookup URL:" } else { "URL гео-определения:" }.to_string(), s.geo_url.clone()),
            (if is_en { "Speedtest URL:" } else { "URL спидтеста:" }.to_string(), s.speedtest_url.clone()),
            (if is_en { "Timeout (ms):" } else { "Таймаут проверки (мс):" }.to_string(), format!("{} ms", s.test_timeout_ms)),
            (if is_en { "Test type:" } else { "Тип теста:" }.to_string(), s.test_type.clone()),
        ],
        7 => vec![
            (if is_en { "Log level:" } else { "Уровень логирования:" }.to_string(), s.log_level.clone()),
            (if is_en { "Log save path:" } else { "Путь к логам:" }.to_string(), s.log_save_path.clone()),
        ],
        8 => vec![
            (if is_en { "uTLS fingerprint:" } else { "uTLS фингерпринт:" }.to_string(), s.utls_fingerprint.clone()),
            (if is_en { "uTLS min version:" } else { "uTLS мин. версия:" }.to_string(), s.utls_min_version.clone()),
            (if is_en { "Autoupdate subs:" } else { "Автообновление подписок:" }.to_string(), s.autoupdate_subs_default.to_string()),
            (if is_en { "Autoupdate resources:" } else { "Автообновление ресурсов:" }.to_string(), s.autoupdate_resources.to_string()),
        ],
        _ => Vec::new(),
    }
}

fn toggle_setting_item(s: &mut AppSettings, cat_idx: usize, item_idx: usize) {
    match (cat_idx, item_idx) {
        // Interface
        (0, 0) => s.language = if s.language == "en" { "ru".to_string() } else { "en".to_string() },
        (0, 1) => s.theme = if s.theme == "dark" { "light".to_string() } else { "dark".to_string() },
        (0, 2) => s.show_icons = !s.show_icons,
        (0, 3) => s.show_country = !s.show_country,
        (0, 4) => s.display_address = !s.display_address,

        // Connection
        (1, 0) => s.connection_mode = if s.connection_mode == "Proxy" { "VPN".to_string() } else { "Proxy".to_string() },
        (1, 1) => s.stack = match s.stack.as_str() {
            "mixed" => "gvisor".to_string(),
            "gvisor" => "system".to_string(),
            _ => "mixed".to_string(),
        },

        // Core
        (2, 0) => s.mux_protocol = match s.mux_protocol.as_str() {
            "h2mux" => "yamux".to_string(),
            "yamux" => "s2mux".to_string(),
            _ => "h2mux".to_string(),
        },
        (2, 1) => s.memory_limit_mb = match s.memory_limit_mb {
            256 => 512,
            512 => 1024,
            1024 => 2048,
            _ => 256,
        },
        (2, 2) => s.cert_validation = !s.cert_validation,

        // Income (Входящие)
        (3, 0) => s.income_local_proxy_in_vpn = !s.income_local_proxy_in_vpn,
        (3, 1) => s.income_turn_off_udp = !s.income_turn_off_udp,
        (3, 2) => s.income_local_ip = if s.income_local_ip == "127.0.0.1" { "0.0.0.0".to_string() } else { "127.0.0.1".to_string() },
        (3, 3) => s.income_local_port = match s.income_local_port {
            20808 => 10808,
            10808 => 7890,
            7890 => 2080,
            _ => 20808,
        },
        (3, 4) => {
            s.income_username = if s.income_username.is_empty() { "admin".to_string() } else { "".to_string() };
        }
        (3, 5) => {
            s.income_password = if s.income_password.is_empty() { "123456".to_string() } else { "".to_string() };
        }
        (3, 6) => s.income_http_proxy_in_vpn = !s.income_http_proxy_in_vpn,
        (3, 7) => s.income_bypass = !s.income_bypass,
        (3, 8) => s.income_hard_route = !s.income_hard_route,
        (3, 9) => {
            s.income_access_from_lan = !s.income_access_from_lan;
            if s.income_access_from_lan {
                s.income_local_ip = "0.0.0.0".to_string();
            } else {
                s.income_local_ip = "127.0.0.1".to_string();
            }
        }

        // Routing (Маршрутизация)
        (4, 0) => s.routing_apps = !s.routing_apps,
        (4, 1) => s.routing_unknown_tun_traffic = match s.routing_unknown_tun_traffic.as_str() {
            "Block" => "Proxy".to_string(),
            "Proxy" => "Direct".to_string(),
            _ => "Block".to_string(),
        },
        (4, 2) => s.routing_system_dns_tun = match s.routing_system_dns_tun.as_str() {
            "Proxy" => "Bypass".to_string(),
            _ => "Proxy".to_string(),
        },
        (4, 3) => s.routing_dns_tun = !s.routing_dns_tun,
        (4, 4) => s.routing_dot_tun = !s.routing_dot_tun,
        (4, 5) => s.routing_doh_tun = !s.routing_doh_tun,
        (4, 6) => s.routing_bypass_lan = !s.routing_bypass_lan,
        (4, 7) => s.routing_bypass_lan_in_core = !s.routing_bypass_lan_in_core,
        (4, 8) => s.routing_dpi = !s.routing_dpi,
        (4, 9) => s.routing_resolve_destination = !s.routing_resolve_destination,
        (4, 10) => s.routing_ipv6_route = !s.routing_ipv6_route,
        (4, 11) => s.routing_rules_source = match s.routing_rules_source.as_str() {
            "official" => "Loyalsoldier".to_string(),
            "Loyalsoldier" => "Chocolate4U".to_string(),
            "Chocolate4U" => "savely-krasovsky".to_string(),
            "savely-krasovsky" => "ITDog".to_string(),
            "ITDog" => "Loyalsoldier-dat".to_string(),
            "Loyalsoldier-dat" => "runetfreedom".to_string(),
            "runetfreedom" => "custom".to_string(),
            _ => "official".to_string(),
        },
        (4, 12) => s.routing_autoupdate_period_days = match s.routing_autoupdate_period_days {
            1 => 3,
            3 => 7,
            7 => 14,
            14 => 30,
            _ => 1,
        },

        // DNS
        (5, 0) => s.dns_routing_enabled = !s.dns_routing_enabled,
        (5, 1) => s.fake_dns = !s.fake_dns,
        (5, 2) => s.direct_dns = match s.direct_dns.as_str() {
            "77.88.8.8" => "1.1.1.1".to_string(),
            "1.1.1.1" => "8.8.8.8".to_string(),
            _ => "77.88.8.8".to_string(),
        },
        (5, 3) => s.proxy_dns = match s.proxy_dns.as_str() {
            "https://1.1.1.1/dns-query" => "https://dns.google/dns-query".to_string(),
            "https://dns.google/dns-query" => "https://77.88.8.8/dns-query".to_string(),
            _ => "https://1.1.1.1/dns-query".to_string(),
        },
        (5, 4) => {},
        (5, 5) => s.enable_ipv6 = !s.enable_ipv6,
        (5, 6) => s.geo_resource = if s.geo_resource == "geoip.db" { "geosite.db".to_string() } else { "geoip.db".to_string() },

        // Tests
        (6, 0) => {},
        (6, 1) => {},
        (6, 2) => {},
        (6, 3) => s.test_timeout_ms = match s.test_timeout_ms {
            1000 => 2000,
            2000 => 3000,
            3000 => 5000,
            5000 => 10000,
            _ => 1000,
        },
        (6, 4) => s.test_type = match s.test_type.as_str() {
            "HTTP" => "TCP".to_string(),
            "TCP" => "RTT".to_string(),
            "RTT" => "GET".to_string(),
            "GET" => "HEAD".to_string(),
            _ => "HTTP".to_string(),
        },

        // Logging
        (7, 0) => s.log_level = match s.log_level.as_str() {
            "trace" => "debug".to_string(),
            "debug" => "info".to_string(),
            "info" => "warn".to_string(),
            "warn" => "error".to_string(),
            _ => "trace".to_string(),
        },
        (7, 1) => {},

        // Other
        (8, 0) => s.utls_fingerprint = match s.utls_fingerprint.as_str() {
            "chrome" => "firefox".to_string(),
            "firefox" => "safari".to_string(),
            "safari" => "randomized".to_string(),
            _ => "chrome".to_string(),
        },
        (8, 1) => s.utls_min_version = if s.utls_min_version == "1.2" { "1.3".to_string() } else { "1.2".to_string() },
        (8, 2) => s.autoupdate_subs_default = !s.autoupdate_subs_default,
        (8, 3) => s.autoupdate_resources = !s.autoupdate_resources,

        _ => {}
    }
}

pub fn print_flat_settings(s: &AppSettings) {
    let is_en = s.language == "en";
    let lang_display = if is_en { "English (EN)" } else { "Русский (RU)" };
    if is_en {
        println!("============================================================");
        println!("             NekoBoxForPC: SYSTEM SETTINGS                  ");
        println!("============================================================");
        println!("[*] Interface  - Language: {}, Theme: {}, Flags: {}, Icons: {}", lang_display, s.theme, s.show_country, s.show_icons);
        println!("[*] Connection - Mode: {}, Stack: {}", s.connection_mode, s.stack);
        println!("[*] Core       - Mux: {}, Memory limit: {} MB, SSL certs: {}", s.mux_protocol, s.memory_limit_mb, s.cert_validation);
        println!("[*] Income     - Local proxy in VPN: {}, IP: {}, Port: {}, Access from LAN: {}", s.income_local_proxy_in_vpn, s.income_local_ip, s.income_local_port, s.income_access_from_lan);
        println!("[*] Routing    - Unknown TUN: {}, DNS TUN: {}, Bypass LAN: {}, Rules source: {}", s.routing_unknown_tun_traffic, s.routing_dns_tun, s.routing_bypass_lan, s.routing_rules_source);
        println!("[*] DNS        - FakeDNS: {}, Direct: {}, DoH: {}", s.fake_dns, s.direct_dns, s.proxy_dns);
        println!("[*] Tests      - URL: {}, Timeout: {} ms, Type: {}", s.test_url, s.test_timeout_ms, s.test_type);
        println!("[*] Logging    - Level: {}, Path: {}", s.log_level, s.log_save_path);
        println!("[*] Other      - uTLS: {}, Autoupdate: {}", s.utls_fingerprint, s.autoupdate_subs_default);
        println!("============================================================");
    } else {
        println!("============================================================");
        println!("             NekoBoxForPC: НАСТРОЙКИ СИСТЕМЫ               ");
        println!("============================================================");
        println!("[*] Интерфейс   - Язык: {}, Тема: {}, Флаги: {}, Иконки: {}", lang_display, s.theme, s.show_country, s.show_icons);
        println!("[*] Подключение - Режим: {}, Стек: {}", s.connection_mode, s.stack);
        println!("[*] Ядро        - Mux: {}, Лимит: {} MB, Проверка SSL: {}", s.mux_protocol, s.memory_limit_mb, s.cert_validation);
        println!("[*] Входящие    - Локальный прокси в VPN: {}, IP: {}, Порт: {}, Доступ из LAN: {}", s.income_local_proxy_in_vpn, s.income_local_ip, s.income_local_port, s.income_access_from_lan);
        println!("[*] Маршрутиз.  - Неизвестный TUN: {}, DNS TUN: {}, Обход LAN: {}, Источник: {}", s.routing_unknown_tun_traffic, s.routing_dns_tun, s.routing_bypass_lan, s.routing_rules_source);
        println!("[*] DNS         - FakeDNS: {}, Прямой: {}, DoH: {}", s.fake_dns, s.direct_dns, s.proxy_dns);
        println!("[*] Тесты       - URL: {}, Таймаут: {} ms, Тип: {}", s.test_url, s.test_timeout_ms, s.test_type);
        println!("[*] Логирование - Уровень: {}, Путь: {}", s.log_level, s.log_save_path);
        println!("[*] Прочее      - uTLS: {}, Автообновление: {}", s.utls_fingerprint, s.autoupdate_subs_default);
        println!("============================================================");
    }
}