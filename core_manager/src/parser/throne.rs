//! Парсер Throne конфигураций на Rust.

use crate::models::{ProfileConfig, ProtocolType};
use crate::parser::{ConfigParser, ParseResult};

pub struct ThroneParser;

impl ConfigParser for ThroneParser {
    fn format_name(&self) -> &'static str {
        "throne"
    }

    fn parse(&self, content: &str) -> ParseResult {
        let text = content.trim();
        let mut result = ParseResult::new(self.format_name());

        if text.starts_with('{') && text.ends_with('}') {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(text) {
                result.raw_config = Some(json_val.clone());
                if let Some(servers) = json_val.get("servers").and_then(|s| s.as_array()) {
                    for item in servers {
                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("Throne Proxy");
                        let server = item.get("server").and_then(|s| s.as_str()).unwrap_or("127.0.0.1");
                        let port = item.get("port").and_then(|p| p.as_u64()).unwrap_or(443) as u16;

                        let p = ProfileConfig::new(name, ProtocolType::Vless, server, port);
                        result.profiles.push(p);
                    }
                    return result;
                }
            }
        }

        for line in text.lines() {
            let l = line.trim();
            if l.starts_with("throne://") {
                if let Ok(url) = url::Url::parse(l) {
                    let host = url.host_str().unwrap_or("127.0.0.1");
                    let port = url.port().unwrap_or(443);
                    let name = url.fragment().unwrap_or("Throne Node");

                    let p = ProfileConfig::new(name, ProtocolType::Vless, host, port);
                    result.profiles.push(p);
                }
            }
        }

        result
    }
}
