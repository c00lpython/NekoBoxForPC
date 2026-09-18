//! Модуль автоопределения и парсинга прокси-конфигураций.

pub mod clash;
pub mod happ;
pub mod incy;
pub mod singbox;
pub mod throne;
pub mod v2ray;
pub mod wireguard;
pub mod xray;

use crate::models::ProfileConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParseResult {
    pub profiles: Vec<ProfileConfig>,
    pub format_name: String,
    pub raw_config: Option<serde_json::Value>,
    pub errors: Vec<String>,
}

impl ParseResult {
    pub fn new(format_name: impl Into<String>) -> Self {
        Self {
            profiles: Vec::new(),
            format_name: format_name.into(),
            raw_config: None,
            errors: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        !self.profiles.is_empty() && self.errors.is_empty()
    }
}

pub trait ConfigParser {
    fn format_name(&self) -> &'static str;
    fn parse(&self, content: &str) -> ParseResult;
}

/// Автоматическое определение формата конфигурации по содержимому.
pub fn detect_config_format(content: &str) -> &'static str {
    let text = content.trim();
    if text.is_empty() {
        return "unknown";
    }

    // 1. По первому URI протоколу
    if let Some(first_line) = text.lines().next() {
        let line = first_line.trim().to_lowercase();
        if line.starts_with("vmess://")
            || line.starts_with("vless://")
            || line.starts_with("trojan://")
            || line.starts_with("ss://")
            || line.starts_with("shadowsocks://")
            || line.starts_with("hysteria2://")
            || line.starts_with("hy2://")
            || line.starts_with("tuic://")
        {
            return "v2ray";
        } else if line.starts_with("happ://") || line.starts_with("hiddify://") {
            return "happ";
        } else if line.starts_with("incy://") {
            return "incy";
        } else if line.starts_with("throne://") {
            return "throne";
        }
    }

    // 2. По структуре WireGuard / AmneziaWG (.conf INI)
    let lower_text = text.to_lowercase();
    if lower_text.contains("[interface]") && lower_text.contains("[peer]") {
        return "wireguard";
    }

    // 3. По структуре JSON
    if text.starts_with('{') && text.ends_with('}') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(obj) = v.as_object() {
                if obj.contains_key("outbounds") {
                    if let Some(outbounds) = obj.get("outbounds").and_then(|o| o.as_array()) {
                        if let Some(first) = outbounds.first().and_then(|f| f.as_object()) {
                            if first.contains_key("type") {
                                return "sing-box";
                            } else if first.contains_key("protocol") {
                                if lower_text.contains("reality")
                                    || lower_text.contains("vision")
                                    || lower_text.contains("mux")
                                {
                                    return "xray";
                                }
                                return "v2ray";
                            }
                        }
                    }
                    return "sing-box";
                } else if obj.contains_key("nodes") {
                    return "incy";
                } else if obj.contains_key("servers") {
                    return "throne";
                } else if obj.contains_key("profiles") {
                    return "happ";
                }
            }
        }
    }

    // 4. По структуре YAML (Clash)
    if text.contains("proxies:") || text.contains("proxy-groups:") || text.contains("rules:") {
        if serde_yaml::from_str::<serde_yaml::Value>(text).is_ok() {
            return "clash";
        }
    }

    "unknown"
}

/// Главная точка входа универсального парсера.
pub fn parse_config(content: &str) -> ParseResult {
    let format = detect_config_format(content);
    match format {
        "v2ray" => v2ray::V2RayParser.parse(content),
        "clash" => clash::ClashParser.parse(content),
        "sing-box" => singbox::SingBoxParser.parse(content),
        "xray" => xray::XrayParser.parse(content),
        "happ" => happ::HappParser.parse(content),
        "incy" => incy::IncyParser.parse(content),
        "throne" => throne::ThroneParser.parse(content),
        "wireguard" => wireguard::WireguardParser.parse(content),
        _ => {
            let mut res = ParseResult::new(format);
            res.errors.push(format!("Неизвестный или неподдерживаемый формат: '{}'", format));
            res
        }
    }
}
