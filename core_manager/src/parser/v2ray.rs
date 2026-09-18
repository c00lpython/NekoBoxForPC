//! Парсер конфигураций формата V2Ray на Rust (URI ссылки и JSON).

use crate::models::{ProfileConfig, ProtocolType};
use crate::parser::{ConfigParser, ParseResult};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use url::Url;

pub struct V2RayParser;

impl ConfigParser for V2RayParser {
    fn format_name(&self) -> &'static str {
        "v2ray"
    }

    fn parse(&self, content: &str) -> ParseResult {
        let text = content.trim();
        let mut result = ParseResult::new(self.format_name());

        if text.is_empty() {
            result.errors.push("Пустой текст V2Ray конфигурации".into());
            return result;
        }

        // 1. Попытка разобрать как V2Ray JSON
        if text.starts_with('{') && text.ends_with('}') {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(text) {
                result.raw_config = Some(json_val.clone());
                if let Some(outbounds) = json_val.get("outbounds").and_then(|o| o.as_array()) {
                    for item in outbounds {
                        let proto_str = item.get("protocol").and_then(|p| p.as_str()).unwrap_or("");
                        if ["freedom", "blackhole", "dns"].contains(&proto_str) {
                            continue;
                        }

                        let tag = item.get("tag").and_then(|t| t.as_str()).unwrap_or("v2ray-proxy");
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

                        let protocol = match proto_str {
                            "vmess" => ProtocolType::Vmess,
                            "vless" => ProtocolType::Vless,
                            "trojan" => ProtocolType::Trojan,
                            "shadowsocks" => ProtocolType::Shadowsocks,
                            _ => ProtocolType::Vless,
                        };

                        let mut p = ProfileConfig::new(tag, protocol, server, port);
                        p.settings = serde_json::json!({
                            "v2ray_settings": settings,
                            "stream_settings": stream
                        });
                        result.profiles.push(p);
                    }
                    return result;
                }
            }
        }

        // 2. Попытка разобрать построчно URI ссылки
        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }

            if let Some(profile) = parse_v2ray_uri(l) {
                result.profiles.push(profile);
            }
        }

        result
    }
}

pub fn parse_v2ray_uri(uri: &str) -> Option<ProfileConfig> {
    if uri.starts_with("vless://") {
        parse_vless_uri(uri)
    } else if uri.starts_with("vmess://") {
        parse_vmess_uri(uri)
    } else if uri.starts_with("trojan://") {
        parse_trojan_uri(uri)
    } else if uri.starts_with("ss://") || uri.starts_with("shadowsocks://") {
        parse_ss_uri(uri)
    } else if uri.starts_with("hysteria2://") || uri.starts_with("hy2://") {
        parse_hysteria2_uri(uri)
    } else if uri.starts_with("tuic://") {
        parse_tuic_uri(uri)
    } else {
        None
    }
}

fn parse_hysteria2_uri(uri: &str) -> Option<ProfileConfig> {
    let clean_uri = if uri.starts_with("hy2://") {
        uri.replacen("hy2://", "hysteria2://", 1)
    } else {
        uri.to_string()
    };
    let url = Url::parse(&clean_uri).ok()?;
    let pass = url.username();
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(443);
    let name = url.fragment().map(|f| urlencoding_decode(f)).unwrap_or_else(|| format!("Hysteria2 {}", host));

    let mut query_map = serde_json::Map::new();
    query_map.insert("password".into(), pass.into());

    for (k, v) in url.query_pairs() {
        query_map.insert(k.into_owned(), v.into_owned().into());
    }

    let mut p = ProfileConfig::new(name, ProtocolType::Hysteria2, host, port);
    p.settings = serde_json::Value::Object(query_map);
    Some(p)
}

fn parse_tuic_uri(uri: &str) -> Option<ProfileConfig> {
    let url = Url::parse(uri).ok()?;
    let uuid = url.username();
    let password = url.password().unwrap_or("");
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(443);
    let name = url.fragment().map(|f| urlencoding_decode(f)).unwrap_or_else(|| format!("TUIC {}", host));

    let mut query_map = serde_json::Map::new();
    query_map.insert("uuid".into(), uuid.into());
    if !password.is_empty() {
        query_map.insert("password".into(), password.into());
    }

    for (k, v) in url.query_pairs() {
        query_map.insert(k.into_owned(), v.into_owned().into());
    }

    let mut p = ProfileConfig::new(name, ProtocolType::Tuic, host, port);
    p.settings = serde_json::Value::Object(query_map);
    Some(p)
}

fn parse_vless_uri(uri: &str) -> Option<ProfileConfig> {
    let url = Url::parse(uri).ok()?;
    let uuid = url.username();
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(443);
    let name = url.fragment().map(|f| urlencoding_decode(f)).unwrap_or_else(|| format!("VLESS {}", host));

    let mut query_map = serde_json::Map::new();
    query_map.insert("uuid".into(), uuid.into());

    for (k, v) in url.query_pairs() {
        query_map.insert(k.into_owned(), v.into_owned().into());
    }

    let mut p = ProfileConfig::new(name, ProtocolType::Vless, host, port);
    p.settings = serde_json::Value::Object(query_map);
    Some(p)
}

fn parse_vmess_uri(uri: &str) -> Option<ProfileConfig> {
    let raw = &uri[8..];
    let padded = pad_base64(raw);
    let decoded = BASE64.decode(padded).ok()?;
    let json_val: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    let name = json_val.get("ps").and_then(|v| v.as_str()).unwrap_or("VMess Proxy");
    let server = json_val.get("add").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
    let port = json_val
        .get("port")
        .and_then(|v| v.as_u64().map(|p| p as u16).or_else(|| v.as_str()?.parse().ok()))
        .unwrap_or(443);

    let mut p = ProfileConfig::new(name, ProtocolType::Vmess, server, port);
    p.settings = json_val;
    Some(p)
}

fn parse_trojan_uri(uri: &str) -> Option<ProfileConfig> {
    let url = Url::parse(uri).ok()?;
    let pass = url.username();
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(443);
    let name = url.fragment().map(|f| urlencoding_decode(f)).unwrap_or_else(|| format!("Trojan {}", host));

    let mut query_map = serde_json::Map::new();
    query_map.insert("password".into(), pass.into());

    for (k, v) in url.query_pairs() {
        query_map.insert(k.into_owned(), v.into_owned().into());
    }

    let mut p = ProfileConfig::new(name, ProtocolType::Trojan, host, port);
    p.settings = serde_json::Value::Object(query_map);
    Some(p)
}

fn parse_ss_uri(uri: &str) -> Option<ProfileConfig> {
    let clean = uri.replace("shadowsocks://", "ss://");
    let url = Url::parse(&clean).ok()?;
    let name = url.fragment().map(|f| urlencoding_decode(f)).unwrap_or_else(|| "Shadowsocks Proxy".into());

    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(8388);
    let userinfo = url.username();

    let (method, password) = if userinfo.contains(':') {
        let mut parts = userinfo.splitn(2, ':');
        (parts.next().unwrap_or("aes-256-gcm").to_string(), parts.next().unwrap_or("").to_string())
    } else if let Ok(decoded) = BASE64.decode(pad_base64(userinfo)) {
        let dec_str = String::from_utf8_lossy(&decoded);
        if dec_str.contains(':') {
            let mut parts = dec_str.splitn(2, ':');
            (parts.next().unwrap_or("aes-256-gcm").to_string(), parts.next().unwrap_or("").to_string())
        } else {
            ("aes-256-gcm".to_string(), dec_str.to_string())
        }
    } else {
        ("aes-256-gcm".to_string(), userinfo.to_string())
    };

    let mut p = ProfileConfig::new(name, ProtocolType::Shadowsocks, host, port);
    p.settings = serde_json::json!({
        "method": method,
        "password": password
    });
    Some(p)
}

fn pad_base64(s: &str) -> String {
    let rem = s.len() % 4;
    if rem > 0 {
        format!("{}{}", s, "=".repeat(4 - rem))
    } else {
        s.to_string()
    }
}

fn urlencoding_decode(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(k, _)| k.into_owned())
        .collect::<Vec<_>>()
        .join("")
}
