//! Интерактивный терминальный пейджер для постраничного просмотра списка узлов.
//! Поддерживает монолитный CLI-режим (без скролла терминала, с очисткой экрана),
//! переключение страниц с шагом +2 / -2 и отображение диапазона соседних страниц ±2.

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

#[derive(Debug, Clone)]
pub struct PagerItem {
    pub index: usize,
    pub group: String,
    pub name: String,
    pub protocol: String,
    pub latency_ms: Option<u64>,
    pub is_picked: bool,
}

/// Вычисляет следующую страницу при шаге вперед с ограничением по общему числу страниц.
pub fn calculate_page_step_forward(current_page: usize, total_pages: usize, step: usize) -> usize {
    if total_pages == 0 {
        return 0;
    }
    (current_page + step).min(total_pages.saturating_sub(1))
}

/// Вычисляет предыдущую страницу при шаге назад без переполнения вниз.
pub fn calculate_page_step_backward(current_page: usize, step: usize) -> usize {
    current_page.saturating_sub(step)
}

/// Форматирует компактную полосу пагинации с окном ±2 страницы вокруг текущей.
pub fn format_pagination_bar(current_page: usize, total_pages: usize) -> String {
    if total_pages == 0 {
        return "[ 0/0 ]".to_string();
    }
    if total_pages == 1 {
        return "[ (1) ]".to_string();
    }

    let curr_one_based = current_page + 1;
    let min_p = curr_one_based.saturating_sub(2).max(1);
    let max_p = (curr_one_based + 2).min(total_pages);

    let mut parts = Vec::new();
    if min_p > 1 {
        parts.push("..".to_string());
    }
    for p in min_p..=max_p {
        if p == curr_one_based {
            parts.push(format!("({})", p));
        } else {
            parts.push(p.to_string());
        }
    }
    if max_p < total_pages {
        parts.push("..".to_string());
    }

    format!("[ {} ]", parts.join(" "))
}

/// Безопасно усекает строку до заданной ширины, чтобы избежать нежелательного переноса строк в терминале.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len <= 1 {
        "…".to_string()
    } else {
        let mut res: String = s.chars().take(max_len.saturating_sub(1)).collect();
        res.push('…');
        res
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerSortMode {
    Default,
    PingAsc,
    PingDesc,
    NameAsc,
    GroupAsc,
    ProtocolAsc,
}

impl PagerSortMode {
    pub fn next(&self) -> Self {
        match self {
            PagerSortMode::Default => PagerSortMode::PingAsc,
            PagerSortMode::PingAsc => PagerSortMode::PingDesc,
            PagerSortMode::PingDesc => PagerSortMode::NameAsc,
            PagerSortMode::NameAsc => PagerSortMode::GroupAsc,
            PagerSortMode::GroupAsc => PagerSortMode::ProtocolAsc,
            PagerSortMode::ProtocolAsc => PagerSortMode::Default,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            PagerSortMode::Default => PagerSortMode::ProtocolAsc,
            PagerSortMode::PingAsc => PagerSortMode::Default,
            PagerSortMode::PingDesc => PagerSortMode::PingAsc,
            PagerSortMode::NameAsc => PagerSortMode::PingDesc,
            PagerSortMode::GroupAsc => PagerSortMode::NameAsc,
            PagerSortMode::ProtocolAsc => PagerSortMode::GroupAsc,
        }
    }

    pub fn display_label(&self, is_en: bool) -> &'static str {
        match (self, is_en) {
            (PagerSortMode::Default, true) => "Default (Index)",
            (PagerSortMode::Default, false) => "По умолчанию (номер)",
            (PagerSortMode::PingAsc, true) => "Ping (Fastest)",
            (PagerSortMode::PingAsc, false) => "Пинг (быстрые)",
            (PagerSortMode::PingDesc, true) => "Ping (Slowest)",
            (PagerSortMode::PingDesc, false) => "Пинг (медленные)",
            (PagerSortMode::NameAsc, true) => "Name (A-Z)",
            (PagerSortMode::NameAsc, false) => "Имя (А-Я)",
            (PagerSortMode::GroupAsc, true) => "Group (A-Z)",
            (PagerSortMode::GroupAsc, false) => "Группа (А-Я)",
            (PagerSortMode::ProtocolAsc, true) => "Protocol",
            (PagerSortMode::ProtocolAsc, false) => "Протокол",
        }
    }
}

pub fn display_paged_list(items: &[PagerItem], title: &str) -> io::Result<()> {
    if !stdout().is_terminal() {
        // Неинтерактивный режим (пайп или CI) - просто выводим весь список
        println!("============================================================");
        println!(" {} (Всего: {})", title, items.len());
        println!("============================================================");
        for item in items {
            print_flat_item(item);
        }
        println!("============================================================");
        return Ok(());
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

    let mut current_page: usize = 0;
    let mut search_query = String::new();
    let mut in_search_mode = false;
    let mut sort_mode = PagerSortMode::Default;

    loop {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let cols_usize = cols as usize;
        let rows_usize = rows as usize;

        // Выделяем строки:
        // Строка 0: заголовок
        // Строка 1: разделитель заголовка
        // Строки 2 .. (rows - 2): элементы страницы
        // Строка rows - 2: разделитель футера
        // Строка rows - 1: футер
        let page_size = rows_usize.saturating_sub(4).max(2);

        // Фильтрация по поисковой строке
        let mut filtered: Vec<&PagerItem> = if search_query.is_empty() {
            items.iter().collect()
        } else {
            let q = search_query.to_lowercase();
            items
                .iter()
                .filter(|it| {
                    it.name.to_lowercase().contains(&q)
                        || it.group.to_lowercase().contains(&q)
                        || it.protocol.to_lowercase().contains(&q)
                })
                .collect()
        };

        // Применяем выбранный режим сортировки
        match sort_mode {
            PagerSortMode::Default => {
                filtered.sort_by_key(|it| it.index);
            }
            PagerSortMode::PingAsc => {
                filtered.sort_by(|a, b| {
                    match (a.latency_ms, b.latency_ms) {
                        (Some(pa), Some(pb)) => pa.cmp(&pb).then_with(|| a.index.cmp(&b.index)),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.index.cmp(&b.index),
                    }
                });
            }
            PagerSortMode::PingDesc => {
                filtered.sort_by(|a, b| {
                    match (a.latency_ms, b.latency_ms) {
                        (Some(pa), Some(pb)) => pb.cmp(&pa).then_with(|| a.index.cmp(&b.index)),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.index.cmp(&b.index),
                    }
                });
            }
            PagerSortMode::NameAsc => {
                filtered.sort_by(|a, b| {
                    a.name
                        .to_lowercase()
                        .cmp(&b.name.to_lowercase())
                        .then_with(|| a.index.cmp(&b.index))
                });
            }
            PagerSortMode::GroupAsc => {
                filtered.sort_by(|a, b| {
                    a.group
                        .to_lowercase()
                        .cmp(&b.group.to_lowercase())
                        .then_with(|| a.index.cmp(&b.index))
                });
            }
            PagerSortMode::ProtocolAsc => {
                filtered.sort_by(|a, b| {
                    a.protocol
                        .to_lowercase()
                        .cmp(&b.protocol.to_lowercase())
                        .then_with(|| a.index.cmp(&b.index))
                });
            }
        }

        let total_items = filtered.len();
        let total_pages = if total_items == 0 {
            1
        } else {
            (total_items + page_size - 1) / page_size
        };

        if current_page >= total_pages {
            current_page = total_pages.saturating_sub(1);
        }

        // 1. Полная очистка экрана и возврат курсора в начало - исключает дублирование кадров в cmd
        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;

        // 2. Отрисовка заголовка (строка 0)
        let title_text = truncate_str(&format!("=== {} ===", title), cols_usize.saturating_sub(2));
        execute!(
            out,
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Cyan),
            Print(title_text),
            ResetColor
        )?;

        // 3. Разделитель заголовка (строка 1)
        let sep_width = cols_usize.saturating_sub(1).min(80);
        execute!(
            out,
            MoveTo(0, 1),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::DarkCyan),
            Print("=".repeat(sep_width)),
            ResetColor
        )?;

        // 4. Отрисовка элементов списка
        let start = current_page * page_size;
        let end = (start + page_size).min(total_items);

        let mut row_cursor: u16 = 2;

        for i in start..end {
            let it = filtered[i];
            let picked_mark = if it.is_picked { "* [PICKED] " } else { "  " };

            let ping_str = match it.latency_ms {
                Some(ms) => format!("{:>4} ms", ms),
                None => " --- ms".to_string(),
            };

            let ping_color = match it.latency_ms {
                Some(ms) if ms < 100 => Color::Green,
                Some(ms) if ms < 300 => Color::Yellow,
                Some(ms) if ms < 800 => Color::Magenta,
                Some(_) => Color::Red,
                None => Color::DarkGrey,
            };

            // Префикс: "  1.(GROUP) * [PICKED] "
            let prefix = format!("{:>3}.({}) {}", it.index, it.group, picked_mark);
            // Суффикс: " | Vless     |  120 ms"
            let proto_str = format!(" | {:<9} | ", truncate_str(&it.protocol, 9));

            // Вычисляем доступную ширину для имени узла, чтобы строка никогда не переносилась
            let fixed_len = prefix.chars().count() + proto_str.chars().count() + ping_str.chars().count() + 2;
            let available_name_len = cols_usize.saturating_sub(fixed_len).max(5);
            let trimmed_name = truncate_str(&it.name, available_name_len);

            execute!(
                out,
                MoveTo(0, row_cursor),
                Clear(ClearType::CurrentLine),
                Print(prefix),
                Print(trimmed_name),
                Print(proto_str),
                SetForegroundColor(ping_color),
                Print(ping_str),
                ResetColor
            )?;

            row_cursor += 1;
        }

        // Очищаем пустые строки до разделителя футера
        let footer_sep_row = rows.saturating_sub(2);
        for r in row_cursor..footer_sep_row {
            execute!(out, MoveTo(0, r), Clear(ClearType::CurrentLine))?;
        }

        // 5. Разделитель над футером
        execute!(
            out,
            MoveTo(0, footer_sep_row),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::DarkGrey),
            Print("-".repeat(sep_width)),
            ResetColor
        )?;

        // 6. Футер управления (строка rows - 1) БЕЗ \r\n, чтобы терминал не скроллился
        let footer_row = rows.saturating_sub(1);
        let lang = crate::storage::ConfigStore::load_or_default(None).data.settings.language;
        let is_en = lang == "en";

        if in_search_mode {
            let search_prompt = if is_en {
                format!("Search (Enter - apply, Esc - cancel): {}_", search_query)
            } else {
                format!("Поиск (Enter - применить, Esc - отмена): {}_", search_query)
            };
            let truncated_prompt = truncate_str(&search_prompt, cols_usize.saturating_sub(2));
            execute!(
                out,
                MoveTo(0, footer_row),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Yellow),
                Print(truncated_prompt),
                ResetColor
            )?;
        } else {
            let pagination_bar = format_pagination_bar(current_page, total_pages);
            let filter_info = if !search_query.is_empty() {
                if is_en {
                    format!(" [Filter: '{}']", search_query)
                } else {
                    format!(" [Фильтр: '{}']", search_query)
                }
            } else {
                "".to_string()
            };

            let sort_label = sort_mode.display_label(is_en);
            let footer_info = if is_en {
                format!(
                    "[←/→] ±1 pg | [PgUp/PgDn] ±2 pg | [↑/W]/[↓/S] Sort: {} | Pg {}/{} {} (Total: {}){} | [/ / F] Search | [Q] Exit",
                    sort_label,
                    current_page + 1,
                    total_pages,
                    pagination_bar,
                    total_items,
                    filter_info
                )
            } else {
                format!(
                    "[←/→] ±1 стр | [PgUp/PgDn] ±2 стр | [↑/W/Ц]/[↓/S/Ы] Сортировка: {} | Стр {}/{} {} (Всего: {}){} | [/ / F / А] Поиск | [Q / Й] Выход",
                    sort_label,
                    current_page + 1,
                    total_pages,
                    pagination_bar,
                    total_items,
                    filter_info
                )
            };
            let truncated_footer = truncate_str(&footer_info, cols_usize.saturating_sub(2));

            execute!(
                out,
                MoveTo(0, footer_row),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Cyan),
                Print(truncated_footer),
                ResetColor
            )?;
        }

        out.flush()?;

        // Ожидание ввода пользователя с поддержкой RU и EN раскладок
        if let Event::Key(key) = event::read()? {
            if !crate::tui::keyboard::is_key_press(&key) {
                continue;
            }
            if in_search_mode {
                match key.code {
                    KeyCode::Enter => {
                        in_search_mode = false;
                        current_page = 0;
                    }
                    KeyCode::Esc => {
                        search_query.clear();
                        in_search_mode = false;
                        current_page = 0;
                    }
                    KeyCode::Backspace => {
                        search_query.pop();
                        current_page = 0;
                    }
                    KeyCode::Char(c) => {
                        search_query.push(c);
                        current_page = 0;
                    }
                    _ => {}
                }
            } else {
                if crate::tui::keyboard::is_exit_key(&key) {
                    break;
                } else if crate::tui::keyboard::is_search_key(&key) {
                    in_search_mode = true;
                } else if crate::tui::keyboard::is_sort_prev(&key) {
                    sort_mode = sort_mode.prev();
                    current_page = 0;
                } else if crate::tui::keyboard::is_sort_next(&key) {
                    sort_mode = sort_mode.next();
                    current_page = 0;
                } else if crate::tui::keyboard::is_step_backward_1(&key) {
                    current_page = calculate_page_step_backward(current_page, 1);
                } else if crate::tui::keyboard::is_step_forward_1(&key) {
                    current_page = calculate_page_step_forward(current_page, total_pages, 1);
                } else if crate::tui::keyboard::is_step_backward_2(&key) {
                    current_page = calculate_page_step_backward(current_page, 2);
                } else if crate::tui::keyboard::is_step_forward_2(&key) {
                    current_page = calculate_page_step_forward(current_page, total_pages, 2);
                } else {
                    match key.code {
                        KeyCode::Home => current_page = 0,
                        KeyCode::End => current_page = total_pages.saturating_sub(1),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_flat_item(it: &PagerItem) {
    let picked_mark = if it.is_picked { "* [PICKED] " } else { "  " };
    let ping_str = match it.latency_ms {
        Some(ms) => format!("{:>4} ms", ms),
        None => " --- ms".to_string(),
    };
    println!(
        "{:>3}.({}) {}{:<30} | {:<10} | {}",
        it.index, it.group, picked_mark, it.name, it.protocol, ping_str
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_page_step_forward_and_backward() {
        let total_pages = 10;
        let mut page = 0;

        // Шаг вперед на +2
        page = calculate_page_step_forward(page, total_pages, 2);
        assert_eq!(page, 2);

        page = calculate_page_step_forward(page, total_pages, 2);
        assert_eq!(page, 4);

        // Плавный шаг на +1
        page = calculate_page_step_forward(page, total_pages, 1);
        assert_eq!(page, 5);

        // Шаг назад на -2
        page = calculate_page_step_backward(page, 2);
        assert_eq!(page, 3);

        // Шаг назад на -2
        page = calculate_page_step_backward(page, 2);
        assert_eq!(page, 1);

        // Шаг назад не уходит ниже 0
        page = calculate_page_step_backward(page, 2);
        assert_eq!(page, 0);

        // Шаг вперед не превышает total_pages - 1
        page = 8;
        page = calculate_page_step_forward(page, total_pages, 2);
        assert_eq!(page, 9);
        page = calculate_page_step_forward(page, total_pages, 2);
        assert_eq!(page, 9);
    }

    #[test]
    fn test_format_pagination_bar() {
        assert_eq!(format_pagination_bar(0, 1), "[ (1) ]");
        assert_eq!(format_pagination_bar(0, 5), "[ (1) 2 3 .. ]");
        assert_eq!(format_pagination_bar(2, 5), "[ 1 2 (3) 4 5 ]");
        assert_eq!(format_pagination_bar(4, 5), "[ .. 3 4 (5) ]");
        assert_eq!(format_pagination_bar(3, 10), "[ .. 2 3 (4) 5 6 .. ]");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 6), "hello…");
        assert_eq!(truncate_str("тест", 3), "те…");
        assert_eq!(truncate_str("", 5), "");
    }

    #[test]
    fn test_pager_sort_mode_cycle() {
        let mut mode = PagerSortMode::Default;
        mode = mode.next();
        assert_eq!(mode, PagerSortMode::PingAsc);
        mode = mode.next();
        assert_eq!(mode, PagerSortMode::PingDesc);
        mode = mode.next();
        assert_eq!(mode, PagerSortMode::NameAsc);
        mode = mode.next();
        assert_eq!(mode, PagerSortMode::GroupAsc);
        mode = mode.next();
        assert_eq!(mode, PagerSortMode::ProtocolAsc);
        mode = mode.next();
        assert_eq!(mode, PagerSortMode::Default);

        // Проверка в обратную сторону (prev)
        mode = mode.prev();
        assert_eq!(mode, PagerSortMode::ProtocolAsc);
        mode = mode.prev();
        assert_eq!(mode, PagerSortMode::GroupAsc);
        mode = mode.prev();
        assert_eq!(mode, PagerSortMode::NameAsc);
        mode = mode.prev();
        assert_eq!(mode, PagerSortMode::PingDesc);
        mode = mode.prev();
        assert_eq!(mode, PagerSortMode::PingAsc);
        mode = mode.prev();
        assert_eq!(mode, PagerSortMode::Default);
    }

    #[test]
    fn test_pager_sort_mode_display_label() {
        assert_eq!(PagerSortMode::Default.display_label(true), "Default (Index)");
        assert_eq!(PagerSortMode::Default.display_label(false), "По умолчанию (номер)");
        assert_eq!(PagerSortMode::PingAsc.display_label(true), "Ping (Fastest)");
        assert_eq!(PagerSortMode::PingAsc.display_label(false), "Пинг (быстрые)");
        assert_eq!(PagerSortMode::PingDesc.display_label(true), "Ping (Slowest)");
        assert_eq!(PagerSortMode::PingDesc.display_label(false), "Пинг (медленные)");
        assert_eq!(PagerSortMode::NameAsc.display_label(true), "Name (A-Z)");
        assert_eq!(PagerSortMode::NameAsc.display_label(false), "Имя (А-Я)");
        assert_eq!(PagerSortMode::GroupAsc.display_label(true), "Group (A-Z)");
        assert_eq!(PagerSortMode::GroupAsc.display_label(false), "Группа (А-Я)");
        assert_eq!(PagerSortMode::ProtocolAsc.display_label(true), "Protocol");
        assert_eq!(PagerSortMode::ProtocolAsc.display_label(false), "Протокол");
    }
}

