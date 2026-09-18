//! Парсер YAML конфигураций Clash / Mihomo на Rust.

use crate::models::{ProfileConfig, ProtocolType};
use crate::parser::{ConfigParser, ParseResult};

pub struct ClashParser;

impl ConfigParser for ClashParser {
    fn format_name(&self) -> &'static str {
        "clash"
    }

    fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new(self.format_name());

        let yaml_val: serde_yaml::Value = match serde_yaml::from_str(content) {
            Ok(v) => v,
            Err(err) => {
                result.errors.push(format!("Ошибка синтаксиса YAML Clash: {}", err));
                return result;
            }
        };

        if let Ok(json_val) = serde_json::to_value(&yaml_val) {
            result.raw_config = Some(json_val.clone());
            if let Some(proxies) = json_val.get("proxies").and_then(|p| p.as_array()) {
                for item in proxies {
                    let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("Clash Proxy");
                    let raw_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("vless").to_lowercase();
                    let server = item.get("server").and_then(|s| s.as_str()).unwrap_or("127.0.0.1");
                    let port = item.get("port").and_then(|p| p.as_u64()).unwrap_or(443) as u16;

                    let protocol = match raw_type.as_str() {
                        "ss" | "shadowsocks" => ProtocolType::Shadowsocks,
                        "vmess" => ProtocolType::Vmess,
                        "vless" => ProtocolType::Vless,
                        "trojan" => ProtocolType::Trojan,
                        "hysteria2" | "hy2" => ProtocolType::Hysteria2,
                        "wireguard" => ProtocolType::Wireguard,
                        "tuic" => ProtocolType::Tuic,
                        "ssh" => ProtocolType::Ssh,
                        "direct" => ProtocolType::Direct,
                        "reject" => ProtocolType::Block,
                        _ => ProtocolType::Vless,
                    };

                    let mut p = ProfileConfig::new(name, protocol, server, port);
                    p.settings = item.clone();
                    result.profiles.push(p);
                }
            }
        }

        result
    }
}
