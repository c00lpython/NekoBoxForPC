//! Парсер Xray JSON конфигураций на Rust.

use crate::models::{ProfileConfig, ProtocolType};
use crate::parser::{ConfigParser, ParseResult};

pub struct XrayParser;

impl ConfigParser for XrayParser {
    fn format_name(&self) -> &'static str {
        "xray"
    }

    fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new(self.format_name());

        let json_val: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(err) => {
                result.errors.push(format!("Ошибка синтаксиса Xray JSON: {}", err));
                return result;
            }
        };

        result.raw_config = Some(json_val.clone());

        if let Some(outbounds) = json_val.get("outbounds").and_then(|o| o.as_array()) {
            for item in outbounds {
                let proto_str = item.get("protocol").and_then(|p| p.as_str()).unwrap_or("vless").to_lowercase();
                let tag = item.get("tag").and_then(|t| t.as_str()).unwrap_or("xray-proxy");

                if ["freedom", "blackhole", "dns"].contains(&proto_str.as_str()) {
                    continue;
                }

                let settings = item.get("settings").cloned().unwrap_or_default();
                let stream = item.get("streamSettings").cloned().unwrap_or_default();

                let mut server = "127.0.0.1".to_string();
                let mut port: u16 = 443;

                if let Some(vnext) = settings.get("vnext").and_then(|v| v.as_array()).and_then(|arr| arr.first()) {
                    if let Some(addr) = vnext.get("address").and_then(|a| a.as_str()) {
                        server = addr.to_string();
                    }
                    if let Some(p) = vnext.get("port").and_then(|p| p.as_u64()) {
                        port = p as u16;
                    }
                }

                let protocol = match proto_str.as_str() {
                    "vless" => ProtocolType::Vless,
                    "vmess" => ProtocolType::Vmess,
                    "trojan" => ProtocolType::Trojan,
                    "shadowsocks" => ProtocolType::Shadowsocks,
                    "wireguard" => ProtocolType::Wireguard,
                    _ => ProtocolType::Vless,
                };

                let mut p = ProfileConfig::new(tag, protocol, server, port);
                p.settings = serde_json::json!({
                    "xray_settings": settings,
                    "stream_settings": stream,
                    "mux": item.get("mux")
                });
                result.profiles.push(p);
            }
        }

        result
    }
}
