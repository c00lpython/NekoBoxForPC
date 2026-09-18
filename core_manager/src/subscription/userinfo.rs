//! Парсер метаданных и лимитов трафика из заголовка `Subscription-Userinfo`.

use serde::{Deserialize, Serialize};

/// Метаинформация об использовании трафика и сроке действия подписки
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SubscriptionUserInfo {
    /// Отправленный трафик (байт)
    pub upload_bytes: u64,
    /// Скачанный трафик (байт)
    pub download_bytes: u64,
    /// Суммарный лимит трафика (байт), 0 если безлимит
    pub total_bytes: u64,
    /// UNIX timestamp (в секундах) окончания подписки (0 если бессрочная)
    pub expire_timestamp: u64,
}

impl SubscriptionUserInfo {
    /// Парсит строку заголовка вида:
    /// `upload=1073741824; download=5368709120; total=107374182400; expire=1735689600`
    pub fn parse(header_value: &str) -> Option<Self> {
        let trimmed = header_value.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut info = Self::default();
        let mut found_any = false;

        for part in trimmed.split(';') {
            let part = part.trim();
            if let Some((key, val)) = part.split_once('=') {
                let key = key.trim().to_ascii_lowercase();
                let val = val.trim();
                match key.as_str() {
                    "upload" => {
                        if let Ok(b) = val.parse::<u64>() {
                            info.upload_bytes = b;
                            found_any = true;
                        }
                    }
                    "download" => {
                        if let Ok(b) = val.parse::<u64>() {
                            info.download_bytes = b;
                            found_any = true;
                        }
                    }
                    "total" => {
                        if let Ok(b) = val.parse::<u64>() {
                            info.total_bytes = b;
                            found_any = true;
                        }
                    }
                    "expire" => {
                        if let Ok(b) = val.parse::<u64>() {
                            info.expire_timestamp = b;
                            found_any = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        if found_any {
            Some(info)
        } else {
            None
        }
    }

    /// Использованный трафик (upload + download)
    pub fn used_bytes(&self) -> u64 {
        self.upload_bytes.saturating_add(self.download_bytes)
    }

    /// Оставшийся трафик (байт)
    pub fn remaining_bytes(&self) -> Option<u64> {
        if self.total_bytes > 0 {
            Some(self.total_bytes.saturating_sub(self.used_bytes()))
        } else {
            None
        }
    }

    /// Процент использования (0..100)
    pub fn used_percent(&self) -> Option<f64> {
        if self.total_bytes > 0 {
            let pct = (self.used_bytes() as f64 / self.total_bytes as f64) * 100.0;
            Some(pct.min(100.0))
        } else {
            None
        }
    }

    /// Форматирование размера данных в читаемый вид (B, KB, MB, GB, TB)
    pub fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        const TB: f64 = GB * 1024.0;

        let b = bytes as f64;
        if b >= TB {
            format!("{:.2} TB", b / TB)
        } else if b >= GB {
            format!("{:.2} GB", b / GB)
        } else if b >= MB {
            format!("{:.2} MB", b / MB)
        } else if b >= KB {
            format!("{:.2} KB", b / KB)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Сводка в виде красивой строки
    pub fn summary(&self) -> String {
        let used = Self::format_bytes(self.used_bytes());
        let total_str = if self.total_bytes > 0 {
            Self::format_bytes(self.total_bytes)
        } else {
            "Безлимит".to_string()
        };

        let pct_str = match self.used_percent() {
            Some(pct) => format!(" ({:.1}%)", pct),
            None => String::new(),
        };

        let expire_str = if self.expire_timestamp > 0 {
            format!("до timestamp: {}", self.expire_timestamp)
        } else {
            "бессрочно".to_string()
        };

        format!(
            "Трафик: {} / {}{}, Истекает: {}",
            used, total_str, pct_str, expire_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subscription_userinfo() {
        let header = "upload=1073741824; download=4294967296; total=107374182400; expire=1735689600";
        let info = SubscriptionUserInfo::parse(header).expect("Failed to parse userinfo");

        assert_eq!(info.upload_bytes, 1073741824); // 1 GB
        assert_eq!(info.download_bytes, 4294967296); // 4 GB
        assert_eq!(info.total_bytes, 107374182400); // 100 GB
        assert_eq!(info.expire_timestamp, 1735689600);
        assert_eq!(info.used_bytes(), 5368709120); // 5 GB
        assert_eq!(info.used_percent(), Some(5.0));
        assert_eq!(SubscriptionUserInfo::format_bytes(info.used_bytes()), "5.00 GB");
    }

    #[test]
    fn test_parse_empty_userinfo() {
        assert!(SubscriptionUserInfo::parse("").is_none());
        assert!(SubscriptionUserInfo::parse("invalid string without equals").is_none());
    }
}
