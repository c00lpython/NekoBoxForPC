//! Парсер конфигураций WireGuard и AmneziaWG (.conf) на Rust.

use crate::models::{ProfileConfig, ProtocolType};
use crate::parser::{ConfigParser, ParseResult};
use serde_json::json;

pub struct WireguardParser;

impl ConfigParser for WireguardParser {
    fn format_name(&self) -> &'static str {
        "wireguard"
    }

    fn parse(&self, content: &str) -> ParseResult {
        let mut result = ParseResult::new(self.format_name());
        let text = content.trim();

        if text.is_empty() {
            result.errors.push("Пустой текст конфигурации WireGuard/AmneziaWG".into());
            return result;
        }

        let mut current_section = "";
        let mut private_key = String::new();
        let mut addresses = Vec::new();
        let mut dns_servers = Vec::new();
        let mut mtu: Option<u64> = None;

        // AmneziaWG специфичные поля
        let mut jc: Option<u64> = None;
        let mut jmin: Option<u64> = None;
        let mut jmax: Option<u64> = None;
        let mut s1: Option<u64> = None;
        let mut s2: Option<u64> = None;
        let mut h1: Option<u64> = None;
        let mut h2: Option<u64> = None;
        let mut h3: Option<u64> = None;
        let mut h4: Option<u64> = None;
        let mut i1: Option<String> = None;

        let mut peer_public_key = String::new();
        let mut endpoint_server = "127.0.0.1".to_string();
        let mut endpoint_port: u16 = 51820;
        let mut allowed_ips = Vec::new();

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = match &line[1..line.len() - 1].to_ascii_lowercase()[..] {
                    "interface" => "interface",
                    "peer" => "peer",
                    _ => "other",
                };
                continue;
            }

            let mut parts = line.splitn(2, '=');
            let key = match parts.next() {
                Some(k) => k.trim().to_ascii_lowercase(),
                None => continue,
            };
            let val = match parts.next() {
                Some(v) => v.trim(),
                None => continue,
            };

            match current_section {
                "interface" => match key.as_str() {
                    "privatekey" => private_key = val.to_string(),
                    "address" => {
                        for addr in val.split(',') {
                            let a = addr.trim();
                            if !a.is_empty() {
                                addresses.push(a.to_string());
                            }
                        }
                    }
                    "dns" => {
                        for d in val.split(',') {
                            let dns_item = d.trim();
                            if !dns_item.is_empty() {
                                dns_servers.push(dns_item.to_string());
                            }
                        }
                    }
                    "mtu" => mtu = val.parse::<u64>().ok(),
                    "jc" => jc = val.parse::<u64>().ok(),
                    "jmin" => jmin = val.parse::<u64>().ok(),
                    "jmax" => jmax = val.parse::<u64>().ok(),
                    "s1" => s1 = val.parse::<u64>().ok(),
                    "s2" => s2 = val.parse::<u64>().ok(),
                    "h1" => h1 = val.parse::<u64>().ok(),
                    "h2" => h2 = val.parse::<u64>().ok(),
                    "h3" => h3 = val.parse::<u64>().ok(),
                    "h4" => h4 = val.parse::<u64>().ok(),
                    "i1" => i1 = Some(val.to_string()),
                    _ => {}
                },
                "peer" => match key.as_str() {
                    "publickey" => peer_public_key = val.to_string(),
                    "endpoint" => {
                        if let Some(colon_pos) = val.rfind(':') {
                            endpoint_server = val[..colon_pos].trim().to_string();
                            if let Ok(p) = val[colon_pos + 1..].trim().parse::<u16>() {
                                endpoint_port = p;
                            }
                        } else {
                            endpoint_server = val.to_string();
                        }
                    }
                    "allowedips" => {
                        for ip in val.split(',') {
                            let ip_item = ip.trim();
                            if !ip_item.is_empty() {
                                allowed_ips.push(ip_item.to_string());
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let is_amnezia = jc.is_some() || jmin.is_some() || jmax.is_some() || s1.is_some() || h1.is_some();
        let protocol = if is_amnezia {
            ProtocolType::Amneziawg
        } else {
            ProtocolType::Wireguard
        };

        let proto_name = if is_amnezia { "AmneziaWG" } else { "WireGuard" };
        let node_name = format!("{} {}", proto_name, endpoint_server);

        let mut p = ProfileConfig::new(node_name, protocol, endpoint_server, endpoint_port);

        let mut settings = json!({
            "private_key": private_key,
            "peer_public_key": peer_public_key,
            "address": addresses,
            "dns": dns_servers,
            "allowed_ips": allowed_ips,
        });

        if let Some(m) = mtu {
            settings["mtu"] = json!(m);
        }
        if let Some(v) = jc {
            settings["jc"] = json!(v);
        }
        if let Some(v) = jmin {
            settings["jmin"] = json!(v);
        }
        if let Some(v) = jmax {
            settings["jmax"] = json!(v);
        }
        if let Some(v) = s1 {
            settings["s1"] = json!(v);
        }
        if let Some(v) = s2 {
            settings["s2"] = json!(v);
        }
        if let Some(v) = h1 {
            settings["h1"] = json!(v);
        }
        if let Some(v) = h2 {
            settings["h2"] = json!(v);
        }
        if let Some(v) = h3 {
            settings["h3"] = json!(v);
        }
        if let Some(v) = h4 {
            settings["h4"] = json!(v);
        }
        if let Some(v) = i1 {
            settings["i1"] = json!(v);
        }

        p.settings = settings;
        result.profiles.push(p);

        result
    }
}
