//! Парсер native sing-box JSON конфигураций на Rust.

use crate::models::{ProfileConfig, ProtocolType};
use crate::parser::{ConfigParser, ParseResult};

pub struct SingBoxParser;

impl ConfigParser for SingBoxParser {
    fn format_name(&self) -> &'static str {
        "sing-box"
    }

    fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new(self.format_name());

        let json_val: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(err) => {
                result.errors.push(format!("Ошибка синтаксиса sing-box JSON: {}", err));
                return result;
            }
        };

        result.raw_config = Some(json_val.clone());

        if let Some(outbounds) = json_val.get("outbounds").and_then(|o| o.as_array()) {
            for item in outbounds {
                let out_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("vless").to_lowercase();
                let tag = item.get("tag").and_then(|t| t.as_str()).unwrap_or("singbox-proxy");

                if ["direct", "block", "dns"].contains(&out_type.as_str()) {
                    continue;
                }

                let server = item.get("server").and_then(|s| s.as_str()).unwrap_or("127.0.0.1");
                let server_port = item.get("server_port").and_then(|p| p.as_u64()).unwrap_or(443) as u16;

                let protocol = match out_type.as_str() {
                    "shadowsocks" => ProtocolType::Shadowsocks,
                    "vmess" => ProtocolType::Vmess,
                    "vless" => ProtocolType::Vless,
                    "trojan" => ProtocolType::Trojan,
                    "hysteria2" => ProtocolType::Hysteria2,
                    "wireguard" => ProtocolType::Wireguard,
                    "tuic" => ProtocolType::Tuic,
                    "ssh" => ProtocolType::Ssh,
                    "direct" => ProtocolType::Direct,
                    "block" => ProtocolType::Block,
                    _ => ProtocolType::Vless,
                };

                let mut p = ProfileConfig::new(tag, protocol, server, server_port);
                p.tag = tag.to_string();
                p.settings = item.clone();
                result.profiles.push(p);
            }
        }

        result
    }
}
