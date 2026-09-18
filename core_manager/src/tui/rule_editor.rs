//! Интерактивный TUI-редактор правил маршрутизации (NewRule / EditRule).
//! Поддерживает раздельный ввод SITE: и IP: с окном выбора пресетов и автодополнений
//! (geosite:, geoip:, rule_set:, regex:, domain_suffix:, ip_cidr:).

use crate::storage::StoredRule;
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

struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(
            out,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Show,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

/// Стандартные пресеты и автодополнения для SITE (домены)
pub const SITE_PRESETS: &[(&str, &str)] = &[
    ("geosite:category-ads-all", "Блокировка рекламы и трекеров"),
    ("geosite:google", "Сервисы Google"),
    ("geosite:youtube", "YouTube видеохостинг"),
    ("geosite:telegram", "Telegram мессенджер"),
    ("geosite:openai", "ChatGPT / OpenAI сервисы"),
    ("geosite:ru", "Зона .RU и сервисы РФ"),
    ("geosite:cn", "Китайские сервисы"),
    ("domain:", "Точное совпадение домена (например domain:example.com)"),
    ("domain_suffix:", "Суффикс домена (например domain_suffix:.ru)"),
    ("domain_regex:", "Регулярное выражение (regex:.*google.*)"),
    ("rule_set:", "Внешний доменный rule-set"),
];

/// Стандартные пресеты и автодополнения для IP адресов
pub const IP_PRESETS: &[(&str, &str)] = &[
    ("geoip:private", "Приватные локальные сети (RFC 1918)"),
    ("geoip:ru", "Российские IP адреса"),
    ("geoip:telegram", "IP адреса Telegram"),
    ("ip_cidr:10.0.0.0/8", "Локальная сеть 10.0.0.0/8"),
    ("ip_cidr:192.168.0.0/16", "Домашняя сеть 192.168.0.0/16"),
    ("ip_cidr:172.16.0.0/12", "Корпоративная сеть 172.16.0.0/12"),
    ("ip_cidr:127.0.0.0/8", "Loopback интерфейс"),
    ("rule_set:", "Внешний IP rule-set"),
];

/// Окно выбора и дополнений для списка (SITE или IP).
/// Позволяет переключать элементы стрелками, выбирать пробелом и дописывать кастомные значения.
pub fn run_preset_selector(
    title: &str,
    presets: &[(&str, &str)],
    current_values: &[String],
) -> io::Result<Vec<String>> {
    if !stdout().is_terminal() {
        return Ok(current_values.to_vec());
    }

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(
        out,
        EnterAlternateScreen,
        Hide,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    let _guard = RawModeGuard;

    run_preset_selector_internal(&mut out, title, presets, current_values)
}

pub fn run_preset_selector_internal(
    out: &mut io::Stdout,
    title: &str,
    presets: &[(&str, &str)],
    current_values: &[String],
) -> io::Result<Vec<String>> {
    let mut selected_items: Vec<String> = current_values.to_vec();
    let mut cursor: usize = 0;
    let mut custom_input = String::new();
    let mut in_custom_mode = false;

    loop {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let cols_u = cols as usize;

        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;

        // Заголовок
        execute!(
            out,
            SetForegroundColor(Color::Cyan),
            Print(format!("=== {} ===\n", title)),
            SetForegroundColor(Color::DarkCyan),
            Print(format!("{}\n", "=".repeat(cols_u.min(80)))),
            ResetColor
        )?;

        // Текущие выбранные значения
        let current_preview = if selected_items.is_empty() {
            "(пусто)".to_string()
        } else {
            selected_items.join(", ")
        };
        execute!(
            out,
            SetForegroundColor(Color::Yellow),
            Print(format!("Выбрано: {}\n\n", current_preview)),
            ResetColor
        )?;

        if in_custom_mode {
            execute!(
                out,
                SetForegroundColor(Color::Green),
                Print(format!("Введите значение (Enter - добавить, Esc - отмена): {}_\n", custom_input)),
                ResetColor
            )?;
        } else {
            writeln!(out, "Доступные шаблоны и автодополнения (Space/Enter - выбрать, 'a' - ввести вручную):")?;

            for (i, (preset_prefix, desc)) in presets.iter().enumerate() {
                let is_current = i == cursor;
                let is_checked = selected_items.iter().any(|s| s == preset_prefix || s.starts_with(preset_prefix));
                let mark = if is_checked { "[X]" } else { "[ ]" };
                let cursor_mark = if is_current { ">" } else { " " };

                if is_current {
                    execute!(
                        out,
                        SetForegroundColor(Color::Yellow),
                        Print(format!(" {} {} {:<26} - {}\n", cursor_mark, mark, preset_prefix, desc)),
                        ResetColor
                    )?;
                } else {
                    writeln!(out, " {} {} {:<26} - {}", cursor_mark, mark, preset_prefix, desc)?;
                }
            }

            // Дополнительный пункт "Ввести вручную"
            let manual_idx = presets.len();
            let is_manual_current = cursor == manual_idx;
            let manual_cursor = if is_manual_current { ">" } else { " " };
            if is_manual_current {
                execute!(
                    out,
                    SetForegroundColor(Color::Green),
                    Print(format!(" {} [+] Ввести кастомное значение вручную...\n", manual_cursor)),
                    ResetColor
                )?;
            } else {
                writeln!(out, " {} [+] Ввести кастомное значение вручную...", manual_cursor)?;
            }
        }

        // Футер
        let footer_row = rows.saturating_sub(1);
        let footer_hint = "[↑/↓/W/S] Навигация | [Space] Выбрать/Снять | [A] Свой ввод | [Enter/Q] Применить и выйти";
        execute!(
            out,
            MoveTo(0, footer_row),
            SetForegroundColor(Color::DarkGrey),
            Print(footer_hint),
            ResetColor
        )?;

        out.flush()?;

        if let Event::Key(key) = event::read()? {
            if !crate::tui::keyboard::is_key_press(&key) {
                continue;
            }

            if in_custom_mode {
                match key.code {
                    KeyCode::Enter => {
                        let trimmed = custom_input.trim().to_string();
                        if !trimmed.is_empty() && !selected_items.contains(&trimmed) {
                            selected_items.push(trimmed);
                        }
                        custom_input.clear();
                        in_custom_mode = false;
                    }
                    KeyCode::Esc => {
                        custom_input.clear();
                        in_custom_mode = false;
                    }
                    KeyCode::Backspace => {
                        custom_input.pop();
                    }
                    KeyCode::Char(c) => {
                        custom_input.push(c);
                    }
                    _ => {}
                }
            } else {
                let max_cursor = presets.len();
                if crate::tui::keyboard::is_exit_key(&key) {
                    break;
                }
                match key.code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Char('ц') | KeyCode::Char('Ц') => {
                        cursor = cursor.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('ы') | KeyCode::Char('Ы') => {
                        if cursor < max_cursor {
                            cursor += 1;
                        }
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Char('ф') | KeyCode::Char('Ф') => {
                        in_custom_mode = true;
                    }
                    KeyCode::Char(' ') => {
                        if cursor < presets.len() {
                            let (prefix, _) = presets[cursor];
                            if prefix.ends_with(':') {
                                // Шаблон с префиксом - открываем ввод с подставленным префиксом
                                custom_input = prefix.to_string();
                                in_custom_mode = true;
                            } else {
                                // Готовый пресет (например geosite:google)
                                if let Some(pos) = selected_items.iter().position(|x| x == prefix) {
                                    selected_items.remove(pos);
                                } else {
                                    selected_items.push(prefix.to_string());
                                }
                            }
                        } else {
                            in_custom_mode = true;
                        }
                    }
                    KeyCode::Enter => {
                        if cursor == max_cursor {
                            in_custom_mode = true;
                        } else if cursor < presets.len() {
                            let (prefix, _) = presets[cursor];
                            if prefix.ends_with(':') {
                                custom_input = prefix.to_string();
                                in_custom_mode = true;
                            } else {
                                if let Some(pos) = selected_items.iter().position(|x| x == prefix) {
                                    selected_items.remove(pos);
                                } else {
                                    selected_items.push(prefix.to_string());
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(selected_items)
}

/// Полноэкранный мастер создания или редактирования правила маршрутизации.
pub fn run_rule_editor_tui(existing: Option<&StoredRule>) -> io::Result<Option<StoredRule>> {
    if !stdout().is_terminal() {
        return Ok(None);
    }

    let mut name = existing.map(|r| r.name.clone()).unwrap_or_else(|| "MyCustomRule".to_string());
    let mut action = existing.map(|r| r.action.clone()).unwrap_or_else(|| "direct".to_string());
    let mut site_list = existing.map(|r| r.site.clone()).unwrap_or_default();
    let mut ip_list = existing.map(|r| r.ip.clone()).unwrap_or_default();
    let enabled = existing.map(|r| r.enabled).unwrap_or(true);

    let actions = ["direct", "proxy", "block", "dns"];
    let mut action_idx = actions.iter().position(|&a| a == action).unwrap_or(0);
    let mut field_idx: usize = 0; // 0: Name, 1: Action, 2: SITE, 3: IP, 4: Save, 5: Cancel

    let mut in_name_input = false;
    let mut temp_name = name.clone();

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(
        out,
        EnterAlternateScreen,
        Hide,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    let _guard = RawModeGuard;

    loop {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let cols_u = cols as usize;

        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;

        let title = if existing.is_some() {
            format!("=== Редактирование правила: '{}' ===", name)
        } else {
            "=== Конструктор нового правила маршрутизации ===".to_string()
        };

        execute!(
            out,
            SetForegroundColor(Color::Cyan),
            Print(format!("{}\n", title)),
            SetForegroundColor(Color::DarkCyan),
            Print(format!("{}\n\n", "=".repeat(cols_u.min(80)))),
            ResetColor
        )?;

        // Отрисовка полей
        // Поле 0: Имя
        let name_marker = if field_idx == 0 { ">" } else { " " };
        let name_color = if field_idx == 0 { Color::Yellow } else { Color::White };
        execute!(
            out,
            SetForegroundColor(name_color),
            Print(format!(" {} 1. Имя правила (Name) : {}\n", name_marker, if in_name_input { format!("{}_", temp_name) } else { name.clone() })),
            ResetColor
        )?;

        // Поле 1: Действие (Action)
        let action_marker = if field_idx == 1 { ">" } else { " " };
        let action_color = if field_idx == 1 { Color::Yellow } else { Color::White };
        execute!(
            out,
            SetForegroundColor(action_color),
            Print(format!(" {} 2. Действие (Action)   : [ {} ] (Space/Enter для переключения)\n", action_marker, actions[action_idx])),
            ResetColor
        )?;

        // Поле 2: SITE (домены)
        let site_marker = if field_idx == 2 { ">" } else { " " };
        let site_color = if field_idx == 2 { Color::Yellow } else { Color::White };
        let site_str = if site_list.is_empty() { "(не задано)" } else { &site_list.join(", ") };
        execute!(
            out,
            SetForegroundColor(site_color),
            Print(format!(" {} 3. Домены (SITE)       : {} (Enter для выбора пресетов)\n", site_marker, site_str)),
            ResetColor
        )?;

        // Поле 3: IP (адреса)
        let ip_marker = if field_idx == 3 { ">" } else { " " };
        let ip_color = if field_idx == 3 { Color::Yellow } else { Color::White };
        let ip_str = if ip_list.is_empty() { "(не задано)" } else { &ip_list.join(", ") };
        execute!(
            out,
            SetForegroundColor(ip_color),
            Print(format!(" {} 4. IP/CIDR (IP)        : {} (Enter для выбора пресетов)\n\n", ip_marker, ip_str)),
            ResetColor
        )?;

        // Поле 4: Кнопка "Сохранить"
        let save_marker = if field_idx == 4 { ">" } else { " " };
        let save_color = if field_idx == 4 { Color::Green } else { Color::DarkGreen };
        execute!(
            out,
            SetForegroundColor(save_color),
            Print(format!(" {} [ СОХРАНИТЬ ПРАВИЛО ]\n", save_marker)),
            ResetColor
        )?;

        // Поле 5: Кнопка "Отмена"
        let cancel_marker = if field_idx == 5 { ">" } else { " " };
        let cancel_color = if field_idx == 5 { Color::Red } else { Color::DarkRed };
        execute!(
            out,
            SetForegroundColor(cancel_color),
            Print(format!(" {} [ Отмена ]\n", cancel_marker)),
            ResetColor
        )?;

        // Футер
        let footer_row = rows.saturating_sub(1);
        let hint = "[↑/↓/W/S] Навигация по полям | [Enter/Space] Редактировать / Выбрать | [Esc/Q] Выход";
        execute!(
            out,
            MoveTo(0, footer_row),
            SetForegroundColor(Color::DarkGrey),
            Print(hint),
            ResetColor
        )?;

        out.flush()?;

        if let Event::Key(key) = event::read()? {
            if !crate::tui::keyboard::is_key_press(&key) {
                continue;
            }

            if in_name_input {
                match key.code {
                    KeyCode::Enter => {
                        let trimmed = temp_name.trim().to_string();
                        if !trimmed.is_empty() {
                            name = trimmed;
                        }
                        in_name_input = false;
                    }
                    KeyCode::Esc => {
                        temp_name = name.clone();
                        in_name_input = false;
                    }
                    KeyCode::Backspace => {
                        temp_name.pop();
                    }
                    KeyCode::Char(c) => {
                        temp_name.push(c);
                    }
                    _ => {}
                }
                continue;
            }

            if crate::tui::keyboard::is_exit_key(&key) {
                return Ok(None);
            }

            match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Char('ц') | KeyCode::Char('Ц') => {
                    field_idx = field_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('ы') | KeyCode::Char('Ы') => {
                    if field_idx < 5 {
                        field_idx += 1;
                    }
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    match field_idx {
                        0 => {
                            // Редактирование имени
                            temp_name = name.clone();
                            in_name_input = true;
                        }
                        1 => {
                            // Переключение действия
                            action_idx = (action_idx + 1) % actions.len();
                            action = actions[action_idx].to_string();
                        }
                        2 => {
                            // Открытие селектора SITE
                            let updated_sites = run_preset_selector_internal(&mut out, "Выбор доменных правил (SITE)", SITE_PRESETS, &site_list)?;
                            site_list = updated_sites;
                        }
                        3 => {
                            // Открытие селектора IP
                            let updated_ips = run_preset_selector_internal(&mut out, "Выбор IP правил (IP)", IP_PRESETS, &ip_list)?;
                            ip_list = updated_ips;
                        }
                        4 => {
                            // Сохранить правило
                            return Ok(Some(StoredRule {
                                name,
                                action,
                                site: site_list,
                                ip: ip_list,
                                enabled,
                            }));
                        }
                        5 => {
                            // Отмена
                            return Ok(None);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
