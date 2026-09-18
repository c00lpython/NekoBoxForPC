//! Модуль селектора профилей и групп с поддержкой диапазонов и инверсии.
//!
//! Поддерживает синтаксис:
//! - Одиночный профиль: `group.config` или `config`
//! - Вся группа: `group_name`
//! - Алфавитные диапазоны: `[a-c],[h-z]`
//! - Инвертированный выбор: `invert [a-c],[h-z]` или `invert group_name`

use crate::storage::{ConfigStore, StoredProfile};

/// Описание одного элемента условия селектора
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorToken {
    /// Одиночный профиль: группа + имя профиля (или None если группа не указана)
    Exact {
        group: Option<String>,
        profile_name: String,
    },
    /// Вся группа целиком
    Group(String),
    /// Диапазон первых символов имени профиля: [start..=end]
    /// Опционально может быть привязан к группе: group.[a-c]
    CharRange {
        group: Option<String>,
        start: char,
        end: char,
    },
    /// Диапазон числовых индексов (1-based): [start..=end] или single number
    IndexRange {
        start: usize,
        end: usize,
    },
}

/// Распарсенный селектор с флагом инверсии
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSelector {
    pub invert: bool,
    pub tokens: Vec<SelectorToken>,
}

impl ParsedSelector {
    /// Парсит строку запроса в структуру ParsedSelector.
    /// Примеры:
    /// - "MyGroup.proxy1"
    /// - "MyGroup"
    /// - "[a-c],[h-z]"
    /// - "[1-3]" или "1-3"
    /// - "invert [a-c],[h-z]"
    /// - "invert MyGroup"
    pub fn parse(input: &str) -> Self {
        let trimmed = input.trim();
        let (invert, remainder) = if let Some(stripped) = trimmed.strip_prefix("invert ") {
            (true, stripped.trim())
        } else if let Some(stripped) = trimmed.strip_prefix("! ") {
            (true, stripped.trim())
        } else if let Some(stripped) = trimmed.strip_prefix('!') {
            (true, stripped.trim())
        } else {
            (false, trimmed)
        };

        // Разбиваем remainder по запятым
        let raw_parts: Vec<&str> = remainder
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut tokens = Vec::new();
        for part in raw_parts {
            // Проверяем формат group.[a-c] или просто [a-c]
            if let Some(token) = Self::parse_part(part) {
                tokens.push(token);
            }
        }

        Self { invert, tokens }
    }

    fn parse_part(part: &str) -> Option<SelectorToken> {
        let (group, expr) = if part.contains('.') {
            let mut s = part.splitn(2, '.');
            let g = s.next()?.trim().to_string();
            let e = s.next()?.trim();
            (Some(g), e)
        } else {
            (None, part)
        };

        let trimmed_expr = expr.trim();

        // Проверяем числовой диапазон [1-5] или 1-5
        let num_cand = if trimmed_expr.starts_with('[') && trimmed_expr.ends_with(']') {
            &trimmed_expr[1..trimmed_expr.len() - 1]
        } else {
            trimmed_expr
        };

        if let Some((s_str, e_str)) = num_cand.split_once('-') {
            if let (Ok(s_num), Ok(e_num)) = (s_str.trim().parse::<usize>(), e_str.trim().parse::<usize>()) {
                let min_n = s_num.min(e_num);
                let max_n = s_num.max(e_num);
                return Some(SelectorToken::IndexRange {
                    start: min_n,
                    end: max_n,
                });
            }
        }

        if let Ok(single_num) = num_cand.trim().parse::<usize>() {
            return Some(SelectorToken::IndexRange {
                start: single_num,
                end: single_num,
            });
        }

        // Проверяем, является ли expr алфавитным диапазоном вида [a-c]
        if trimmed_expr.starts_with('[') && trimmed_expr.ends_with(']') && trimmed_expr.len() >= 5 {
            let inner = &trimmed_expr[1..trimmed_expr.len() - 1]; // "a-c"
            if let Some((start_s, end_s)) = inner.split_once('-') {
                let start = start_s.trim().chars().next()?.to_ascii_lowercase();
                let end = end_s.trim().chars().next()?.to_ascii_lowercase();
                let (min_c, max_c) = if start <= end { (start, end) } else { (end, start) };
                return Some(SelectorToken::CharRange {
                    group,
                    start: min_c,
                    end: max_c,
                });
            }
        }

        // Если есть group и expr: это Exact
        if let Some(g) = group {
            return Some(SelectorToken::Exact {
                group: Some(g),
                profile_name: expr.to_string(),
            });
        }

        Some(SelectorToken::Exact {
            group: None,
            profile_name: expr.to_string(),
        })
    }

    /// Выполняет выбор профилей из хранилища в соответствии с распарсенным селектором
    /// Возвращает список кортежей: (имя_группы, StoredProfile)
    pub fn select<'a>(&self, store: &'a ConfigStore) -> Vec<(String, StoredProfile)> {
        // 1. Собираем всех кандидатов из хранилища (все доступные профили во всех группах)
        let mut all_profiles: Vec<(String, StoredProfile)> = Vec::new();
        for group in &store.data.groups {
            for prof in &group.profiles {
                all_profiles.push((group.name.clone(), prof.clone()));
            }
        }

        // 2. Определяем, какие профили подпадают под токены
        let mut matched: Vec<(String, StoredProfile)> = Vec::new();

        for (idx, (grp_name, prof)) in all_profiles.iter().enumerate() {
            let mut is_match = false;
            let one_based_idx = idx + 1;
            for token in &self.tokens {
                if Self::matches_token(token, one_based_idx, grp_name, prof, store) {
                    is_match = true;
                    break;
                }
            }

            if is_match {
                matched.push((grp_name.clone(), prof.clone()));
            }
        }

        // 3. Применяем инверсию при необходимости
        if self.invert {
            // Оставляем те, которых нет в matched
            all_profiles
                .into_iter()
                .filter(|(g, p)| !matched.iter().any(|(mg, mp)| mg == g && mp.name == p.name))
                .collect()
        } else {
            matched
        }
    }

    fn matches_token(
        token: &SelectorToken,
        index: usize,
        group_name: &str,
        prof: &StoredProfile,
        store: &ConfigStore,
    ) -> bool {
        match token {
            SelectorToken::IndexRange { start, end } => {
                index >= *start && index <= *end
            }
            SelectorToken::CharRange { group, start, end } => {
                if let Some(expected_group) = group {
                    if !group_name.eq_ignore_ascii_case(expected_group) {
                        return false;
                    }
                }
                if let Some(first_char) = prof.name.chars().next() {
                    let c = first_char.to_ascii_lowercase();
                    c >= *start && c <= *end
                } else {
                    false
                }
            }
            SelectorToken::Group(grp) => group_name.eq_ignore_ascii_case(grp),
            SelectorToken::Exact { group, profile_name } => {
                // Если указана группа:
                if let Some(g) = group {
                    group_name.eq_ignore_ascii_case(g) && prof.name.eq_ignore_ascii_case(profile_name)
                } else {
                    // Группа не указана.
                    // Если profile_name совпадает с именем существующей группы — считаем, что выбрана вся группа!
                    if store.get_group(profile_name).is_some() {
                        group_name.eq_ignore_ascii_case(profile_name)
                    } else {
                        // Иначе проверяем совпадение имени профиля
                        prof.name.eq_ignore_ascii_case(profile_name)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProtocolType;

    fn create_test_store() -> ConfigStore {
        let mut store = ConfigStore::load_or_default(None);
        store.data.groups.clear();

        let mut g1 = crate::storage::StoredGroup::new_manual("GroupA");
        let mut g2 = crate::storage::StoredGroup::new_manual("GroupB");

        for name in &["alpha", "bravo", "charlie", "delta", "hotel", "zulu"] {
            g1.profiles.push(StoredProfile {
                id: name.to_string(),
                name: name.to_string(),
                protocol: ProtocolType::Vmess,
                server: "1.2.3.4".to_string(),
                server_port: 443,
                settings: serde_json::json!({}),
                tag: name.to_string(),
                last_ping_ms: None,
            });
        }

        for name in &["beta", "gamma", "omega"] {
            g2.profiles.push(StoredProfile {
                id: name.to_string(),
                name: name.to_string(),
                protocol: ProtocolType::Vless,
                server: "5.6.7.8".to_string(),
                server_port: 8443,
                settings: serde_json::json!({}),
                tag: name.to_string(),
                last_ping_ms: None,
            });
        }

        store.data.groups.push(g1);
        store.data.groups.push(g2);
        store
    }

    #[test]
    fn test_exact_profile_selector() {
        let store = create_test_store();
        let sel = ParsedSelector::parse("GroupA.alpha");
        let res = sel.select(&store);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.name, "alpha");
    }

    #[test]
    fn test_group_selector() {
        let store = create_test_store();
        let sel = ParsedSelector::parse("GroupB");
        let res = sel.select(&store);
        assert_eq!(res.len(), 3);
        let names: Vec<String> = res.into_iter().map(|(_, p)| p.name).collect();
        assert_eq!(names, vec!["beta", "gamma", "omega"]);
    }

    #[test]
    fn test_range_selector() {
        let store = create_test_store();
        // [a-c] должен выбрать alpha, bravo, charlie, beta
        let sel = ParsedSelector::parse("[a-c]");
        let res = sel.select(&store);
        let names: Vec<String> = res.into_iter().map(|(_, p)| p.name).collect();
        assert_eq!(names, vec!["alpha", "bravo", "charlie", "beta"]);
    }

    #[test]
    fn test_multi_range_selector() {
        let store = create_test_store();
        // [a-c],[h-z]
        let sel = ParsedSelector::parse("[a-c],[h-z]");
        let res = sel.select(&store);
        let names: Vec<String> = res.into_iter().map(|(_, p)| p.name).collect();
        // GroupA: alpha, bravo, charlie, hotel, zulu (delta пропущена)
        // GroupB: beta, omega (gamma пропущена)
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"hotel".to_string()));
        assert!(names.contains(&"zulu".to_string()));
        assert!(names.contains(&"beta".to_string()));
        assert!(names.contains(&"omega".to_string()));
        assert!(!names.contains(&"delta".to_string()));
        assert!(!names.contains(&"gamma".to_string()));
    }

    #[test]
    fn test_invert_range_selector() {
        let store = create_test_store();
        // invert [a-c],[h-z] -> должны остаться только delta и gamma!
        let sel = ParsedSelector::parse("invert [a-c],[h-z]");
        let res = sel.select(&store);
        let names: Vec<String> = res.into_iter().map(|(_, p)| p.name).collect();
        assert_eq!(names, vec!["delta", "gamma"]);
    }

    #[test]
    fn test_index_range_selector() {
        let store = create_test_store();
        // Тест диапазона по сквозным индексам 1-3
        let sel = ParsedSelector::parse("[1-3]");
        let res = sel.select(&store);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].1.name, "alpha");
        assert_eq!(res[1].1.name, "bravo");
        assert_eq!(res[2].1.name, "charlie");

        // Тест одиночного индекса
        let sel_single = ParsedSelector::parse("2");
        let res_single = sel_single.select(&store);
        assert_eq!(res_single.len(), 1);
        assert_eq!(res_single[0].1.name, "bravo");
    }
}
