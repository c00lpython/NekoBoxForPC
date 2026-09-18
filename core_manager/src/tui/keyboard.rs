//! Модуль кросс-раскладочной нормализации клавиатурного ввода (EN <-> RU).
//! Обеспечивает безотказную работу горячих клавиш (Q/Й, Ctrl+F/Ctrl+А, W/Ц, S/Ы, A/Ф, D/В и т.д.)
//! независимо от активной системной раскладки пользователя.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Нормализует символ русской раскладки ЙЦУКЕН к соответствующему
/// латинскому символу стандартной раскладки QWERTY на той же физической клавише.
pub fn normalize_char(c: char) -> char {
    match c {
        'й' | 'Й' => 'q',
        'ц' | 'Ц' => 'w',
        'у' | 'У' => 'e',
        'к' | 'К' => 'r',
        'е' | 'Е' => 't',
        'н' | 'Н' => 'y',
        'г' | 'Г' => 'u',
        'ш' | 'Ш' => 'i',
        'щ' | 'Щ' => 'o',
        'з' | 'З' => 'p',
        'х' | 'Х' => '[',
        'ъ' | 'Ъ' => ']',
        'ф' | 'Ф' => 'a',
        'ы' | 'Ы' => 's',
        'в' | 'В' => 'd',
        'а' | 'А' => 'f',
        'п' | 'П' => 'g',
        'р' | 'Р' => 'h',
        'о' | 'О' => 'j',
        'л' | 'Л' => 'k',
        'д' | 'Д' => 'l',
        'я' | 'Я' => 'z',
        'ч' | 'Ч' => 'x',
        'с' | 'С' => 'c',
        'м' | 'М' => 'v',
        'и' | 'И' => 'b',
        'т' | 'Т' => 'n',
        'ь' | 'Ь' => 'm',
        'ю' | 'Ю' => '.',
        'б' | 'Б' => ',',
        '.' => '/',
        other => other.to_ascii_lowercase(),
    }
}

/// Распознает команду выхода (Q, q, Й, й, Esc, Ctrl+C, Ctrl+С).
pub fn is_exit_key(key: &KeyEvent) -> bool {
    if key.code == KeyCode::Esc {
        return true;
    }
    if let KeyCode::Char(c) = key.code {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            let n = normalize_char(c);
            return n == 'c';
        }
        let n = normalize_char(c);
        return n == 'q';
    }
    false
}

/// Распознает команду поиска (F, f, А, а, Ctrl+F, Ctrl+А, /, F3).
/// Прямое нажатие F / А предотвращает перехват горячей клавиши терминалом (Windows Terminal Find).
pub fn is_search_key(key: &KeyEvent) -> bool {
    if key.code == KeyCode::F(3) {
        return true;
    }
    if let KeyCode::Char(c) = key.code {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            let n = normalize_char(c);
            return n == 'f';
        }
        let n = normalize_char(c);
        return n == '/' || n == 'f';
    }
    false
}

/// Распознает команду перехода назад на 1 страницу (стрелка Влево, [).
pub fn is_step_backward_1(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Left => true,
        KeyCode::Char(c) => {
            let n = normalize_char(c);
            n == '['
        }
        _ => false,
    }
}

/// Распознает команду перехода вперед на 1 страницу (стрелка Вправо, ]).
pub fn is_step_forward_1(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Right => true,
        KeyCode::Char(c) => {
            let n = normalize_char(c);
            n == ']'
        }
        _ => false,
    }
}

/// Распознает команду смены сортировки вверх / предыдущий режим (стрелка Вверх, W, Ц).
pub fn is_sort_prev(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Up => true,
        KeyCode::Char(c) => {
            let n = normalize_char(c);
            n == 'w'
        }
        _ => false,
    }
}

/// Распознает команду смены сортировки вниз / следующий режим (стрелка Вниз, S, Ы).
pub fn is_sort_next(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Down => true,
        KeyCode::Char(c) => {
            let n = normalize_char(c);
            n == 's'
        }
        _ => false,
    }
}

/// Распознает команду быстрого перехода назад на 2 страницы (PageUp, A, Ф, -, <).
pub fn is_step_backward_2(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::PageUp => true,
        KeyCode::Char(c) => {
            let n = normalize_char(c);
            n == 'a' || c == '-' || c == '<' || c == ',' || c == 'б' || c == 'Б'
        }
        _ => false,
    }
}

/// Распознает команду быстрого перехода вперед на 2 страницы (PageDown, D, В, +, >).
pub fn is_step_forward_2(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::PageDown => true,
        KeyCode::Char(c) => {
            let n = normalize_char(c);
            n == 'd' || c == '+' || c == '>' || c == '.' || c == 'ю' || c == 'Ю'
        }
        _ => false,
    }
}

/// Проверяет, является ли событие нажатием клавиши (KeyEventKind::Press),
/// отсекая лишние события отпускания (Release) на Windows crossterm.
pub fn is_key_press(key: &KeyEvent) -> bool {
    key.kind == crossterm::event::KeyEventKind::Press
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_russian_keys() {
        assert_eq!(normalize_char('й'), 'q');
        assert_eq!(normalize_char('Й'), 'q');
        assert_eq!(normalize_char('а'), 'f');
        assert_eq!(normalize_char('А'), 'f');
        assert_eq!(normalize_char('ц'), 'w');
        assert_eq!(normalize_char('ы'), 's');
        assert_eq!(normalize_char('ф'), 'a');
        assert_eq!(normalize_char('в'), 'd');
        assert_eq!(normalize_char('с'), 'c');
        assert_eq!(normalize_char('Q'), 'q');
        assert_eq!(normalize_char('f'), 'f');
    }

    #[test]
    fn test_is_exit_key_bilingual() {
        let ev_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        assert!(is_exit_key(&ev_q));

        let ev_yo = KeyEvent::new(KeyCode::Char('й'), KeyModifiers::empty());
        assert!(is_exit_key(&ev_yo));

        let ev_yo_upper = KeyEvent::new(KeyCode::Char('Й'), KeyModifiers::empty());
        assert!(is_exit_key(&ev_yo_upper));

        let ev_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        assert!(is_exit_key(&ev_esc));

        let ev_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_exit_key(&ev_ctrl_c));

        let ev_ctrl_c_ru = KeyEvent::new(KeyCode::Char('с'), KeyModifiers::CONTROL);
        assert!(is_exit_key(&ev_ctrl_c_ru));
    }

    #[test]
    fn test_is_search_key_bilingual() {
        let ev_ctrl_f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        assert!(is_search_key(&ev_ctrl_f));

        let ev_ctrl_a_ru = KeyEvent::new(KeyCode::Char('а'), KeyModifiers::CONTROL);
        assert!(is_search_key(&ev_ctrl_a_ru));

        let ev_f_direct = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty());
        assert!(is_search_key(&ev_f_direct));

        let ev_a_direct_ru = KeyEvent::new(KeyCode::Char('а'), KeyModifiers::empty());
        assert!(is_search_key(&ev_a_direct_ru));

        let ev_f3 = KeyEvent::new(KeyCode::F(3), KeyModifiers::empty());
        assert!(is_search_key(&ev_f3));

        let ev_slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty());
        assert!(is_search_key(&ev_slash));
    }

    #[test]
    fn test_arrows_step_single_page() {
        let ev_left = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
        assert!(is_step_backward_1(&ev_left));

        let ev_right = KeyEvent::new(KeyCode::Right, KeyModifiers::empty());
        assert!(is_step_forward_1(&ev_right));

        let ev_open_bracket = KeyEvent::new(KeyCode::Char('['), KeyModifiers::empty());
        assert!(is_step_backward_1(&ev_open_bracket));

        let ev_close_bracket = KeyEvent::new(KeyCode::Char(']'), KeyModifiers::empty());
        assert!(is_step_forward_1(&ev_close_bracket));
    }

    #[test]
    fn test_arrows_sort_modes() {
        let ev_up = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert!(is_sort_prev(&ev_up));

        let ev_down = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert!(is_sort_next(&ev_down));

        let ev_ts = KeyEvent::new(KeyCode::Char('ц'), KeyModifiers::empty());
        assert!(is_sort_prev(&ev_ts));

        let ev_y = KeyEvent::new(KeyCode::Char('ы'), KeyModifiers::empty());
        assert!(is_sort_next(&ev_y));
    }

    #[test]
    fn test_step_two_pages() {
        let ev_pgdn = KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty());
        assert!(is_step_forward_2(&ev_pgdn));

        let ev_pgup = KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty());
        assert!(is_step_backward_2(&ev_pgup));

        let ev_f = KeyEvent::new(KeyCode::Char('ф'), KeyModifiers::empty());
        assert!(is_step_backward_2(&ev_f));
    }

    #[test]
    fn test_is_key_press_filters_release() {
        use crossterm::event::{KeyEventKind, KeyEventState};
        let press = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        let release = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Release,
            state: KeyEventState::empty(),
        };
        let repeat = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Repeat,
            state: KeyEventState::empty(),
        };

        assert!(is_key_press(&press));
        assert!(!is_key_press(&release));
        assert!(!is_key_press(&repeat));
    }
}