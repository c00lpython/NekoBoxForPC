//! Конвертер доменных моделей Rust в sing-box JSON на Rust.

use crate::models::{AppConfig, ProfileConfig, ProtocolType, RoutingMode};
use serde_json::{json, Value};

pub fn to_singbox_json(
    profiles: &[ProfileConfig],
    app_config: Option<&AppConfig>,
    active_profile_id: Option<&str>,
) -> Value {
    let default_app_config = AppConfig::default();
    let config = app_config.unwrap_or(&default_app_config);

    let active_profile = active_profile_id
        .and_then(|id| profiles.iter().find(|p| p.id == id))
        .or_else(|| profiles.first());

    let mut outbounds = Vec::new();

    if let Some(profile) = active_profile {
        outbounds.push(profile_to_outbound(profile, "proxy"));
    } else {
        outbounds.push(json!({
            "type": "direct",
            "tag": "proxy"
        }));
    }

    outbounds.push(json!({"type": "direct", "tag": "direct"}));
    outbounds.push(json!({"type": "block", "tag": "block"}));

    let mut inbounds = vec![json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": if config.inbound.allow_lan { "0.0.0.0" } else { "127.0.0.1" },
        "listen_port": config.inbound.mixed_port,
    })];

    if config.tun.enabled {
        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": config.tun.interface_name,
            "inet4_address": [config.tun.inet4_address],
            "inet6_address": if config.tun.inet6_address.is_empty() { vec![] } else { vec![config.tun.inet6_address.clone()] },
            "auto_route": config.tun.auto_route,
            "strict_route": config.tun.strict_route,
            "stack": match config.tun.stack {
                crate::models::TUNStack::System => "system",
                crate::models::TUNStack::Gvisor => "gvisor",
                crate::models::TUNStack::Mixed => "mixed",
            },
            "mtu": config.tun.mtu,
        }));
    }

    let mut rules = Vec::new();

    match config.routing_mode {
        RoutingMode::Global => rules.push(json!({"outbound": "proxy"})),
        RoutingMode::Direct => rules.push(json!({"outbound": "direct"})),
        RoutingMode::Rule => {
            for rule in &config.rules {
                let mut r = json!({"outbound": rule.outbound});
                if !rule.domains.is_empty() {
                    r["domain"] = json!(rule.domains);
                }
                if !rule.domain_suffixes.is_empty() {
                    r["domain_suffix"] = json!(rule.domain_suffixes);
                }
                if !rule.domain_keywords.is_empty() {
                    r["domain_keyword"] = json!(rule.domain_keywords);
                }
                if !rule.ip_cidrs.is_empty() {
                    r["ip_cidr"] = json!(rule.ip_cidrs);
                }
                if !rule.geoip.is_empty() {
                    r["geoip"] = json!(rule.geoip);
                }
                if !rule.geosite.is_empty() {
                    r["geosite"] = json!(rule.geosite);
                }
                rules.push(r);
            }
        }
    }

    rules.insert(0, json!({"action": "sniff"}));
    rules.insert(1, json!({"protocol": "dns", "action": "hijack-dns"}));

    json!({
        "log": {
            "level": "trace",
            "timestamp": true
        },
        "dns": {
            "servers": [
                {
                    "tag": "local-dns",
                    "address": "local",
                    "detour": "direct"
                },
                {
                    "tag": "remote-dns",
                    "address": "https://1.1.1.1/dns-query",
                    "address_resolver": "local-dns",
                    "detour": "proxy"
                }
            ],
            "rules": [
                {
                    "outbound": ["any"],
                    "server": "local-dns"
                }
            ],
            "strategy": "prefer_ipv4"
        },
        "inbounds": inbounds,
        "outbounds": outbounds,
        "route": {
            "rules": rules,
            "auto_detect_interface": true
        }
    })
}

pub fn profile_to_outbound(profile: &ProfileConfig, tag: &str) -> Value {
    if profile.protocol == ProtocolType::Direct {
        return json!({
            "type": "direct",
            "tag": tag
        });
    }
    if profile.protocol == ProtocolType::Block {
        return json!({
            "type": "block",
            "tag": tag
        });
    }
    if profile.protocol == ProtocolType::Balancer {
        if let Some(nodes) = profile.settings.get("candidates").and_then(|v| v.as_array()) {
            let candidate_tags: Vec<String> = nodes.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
            return json!({
                "type": "urltest",
                "tag": tag,
                "outbounds": candidate_tags,
                "url": "https://www.gstatic.com/generate_204",
                "interval": "3m",
                "tolerance": 50
            });
        } else {
            return json!({
                "type": "direct",
                "tag": tag
            });
        }
    }
    if profile.protocol == ProtocolType::Chain {
        return json!({
            "type": "direct",
            "tag": tag
        });
    }

    let mut obj = json!({
        "type": match profile.protocol {
            ProtocolType::Shadowsocks => "shadowsocks",
            ProtocolType::Vmess => "vmess",
            ProtocolType::Vless => "vless",
            ProtocolType::Trojan => "trojan",
            ProtocolType::Hysteria2 => "hysteria2",
            ProtocolType::Wireguard => "wireguard",
            ProtocolType::Amneziawg => "amneziawg",
            ProtocolType::Tuic => "tuic",
            ProtocolType::Ssh => "ssh",
            ProtocolType::Byedpi => "byedpi",
            ProtocolType::Dns => "dns",
            _ => "direct",
        },
        "tag": tag,
        "server": profile.server,
        "server_port": profile.server_port
    });

    if let Some(map) = profile.settings.as_object() {
        match profile.protocol {
            ProtocolType::Vless => {
                if let Some(uuid) = map.get("uuid").and_then(|v| v.as_str()) {
                    obj["uuid"] = json!(uuid);
                }
                if let Some(flow) = map.get("flow").and_then(|v| v.as_str()) {
                    if !flow.is_empty() {
                        obj["flow"] = json!(flow);
                    }
                }
                obj["packet_encoding"] = json!(map.get("packet_encoding").and_then(|v| v.as_str()).unwrap_or("xudp"));
                let security = map.get("security").and_then(|v| v.as_str()).unwrap_or("");
                if security == "tls" || security == "reality" {
                    let sni = map.get("sni").and_then(|v| v.as_str()).unwrap_or(&profile.server);
                    let mut tls = json!({
                        "enabled": true,
                        "server_name": sni
                    });
                    if security == "reality" {
                        tls["reality"] = json!({
                            "enabled": true,
                            "public_key": map.get("pbk").and_then(|v| v.as_str()).unwrap_or(""),
                            "short_id": map.get("sid").and_then(|v| v.as_str()).unwrap_or("")
                        });
                        let fp = map.get("fp").and_then(|v| v.as_str()).unwrap_or("chrome");
                        tls["utls"] = json!({
                            "enabled": true,
                            "fingerprint": fp
                        });
                    } else if let Some(fp) = map.get("fp").and_then(|v| v.as_str()) {
                        tls["utls"] = json!({
                            "enabled": true,
                            "fingerprint": fp
                        });
                    }
                    obj["tls"] = tls;
                }
            }
            ProtocolType::Vmess => {
                if let Some(uuid) = map.get("uuid").and_then(|v| v.as_str()) {
                    obj["uuid"] = json!(uuid);
                }
                obj["security"] = json!(map.get("security").and_then(|v| v.as_str()).unwrap_or("auto"));
                obj["packet_encoding"] = json!("packetaddr");
            }
            ProtocolType::Trojan => {
                if let Some(pass) = map.get("password").and_then(|v| v.as_str()) {
                    obj["password"] = json!(pass);
                }
            }
            ProtocolType::Shadowsocks => {
                obj["method"] = json!(map.get("method").and_then(|v| v.as_str()).unwrap_or("aes-256-gcm"));
                obj["password"] = json!(map.get("password").and_then(|v| v.as_str()).unwrap_or(""));
            }
            ProtocolType::Hysteria2 => {
                if let Some(pass) = map.get("password").and_then(|v| v.as_str()) {
                    obj["password"] = json!(pass);
                }
                if let Some(sni) = map.get("sni").and_then(|v| v.as_str()) {
                    let insecure = map.get("insecure").and_then(|v| v.as_str()).map(|s| s == "1" || s == "true").unwrap_or(false);
                    obj["tls"] = json!({
                        "enabled": true,
                        "server_name": sni,
                        "insecure": insecure
                    });
                }
            }
            ProtocolType::Tuic => {
                if let Some(uuid) = map.get("uuid").and_then(|v| v.as_str()) {
                    obj["uuid"] = json!(uuid);
                }
                if let Some(pass) = map.get("password").and_then(|v| v.as_str()) {
                    obj["password"] = json!(pass);
                }
                if let Some(cc) = map.get("congestion_control").and_then(|v| v.as_str()) {
                    obj["congestion_control"] = json!(cc);
                }
            }
            ProtocolType::Wireguard | ProtocolType::Amneziawg => {
                let is_awg = profile.protocol == ProtocolType::Amneziawg;
                obj["type"] = json!("wireguard");
                if let Some(pk) = map.get("private_key").and_then(|v| v.as_str()) {
                    obj["private_key"] = json!(pk);
                }
                if let Some(peer_pk) = map.get("peer_public_key").and_then(|v| v.as_str()) {
                    obj["peer_public_key"] = json!(peer_pk);
                }
                if let Some(addrs) = map.get("address").and_then(|v| v.as_array()) {
                    obj["local_address"] = json!(addrs);
                }
                if let Some(mtu) = map.get("mtu") {
                    obj["mtu"] = mtu.clone();
                }
                obj["system_interface"] = json!(false);

                if is_awg {
                    for key in &["jc", "jmin", "jmax", "s1", "s2", "h1", "h2", "h3", "h4"] {
                        if let Some(val) = map.get(*key) {
                            obj[*key] = val.clone();
                        }
                    }
                }
            }
            ProtocolType::Byedpi => {
                obj["type"] = json!("socks");
                obj["server"] = json!(if profile.server.is_empty() { "127.0.0.1" } else { &profile.server });
                obj["server_port"] = json!(if profile.server_port == 0 { 1080 } else { profile.server_port });
            }
            ProtocolType::MasterDnsVPN => {
                obj["type"] = json!("dns");
            }
            ProtocolType::Chain => {
                obj["type"] = json!("direct");
            }
            ProtocolType::Balancer => {
                obj["type"] = json!("urltest");
            }
            _ => {}
        }

        let net_type = map.get("type")
            .or_else(|| map.get("net"))
            .or_else(|| map.get("network"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if net_type == "ws" {
            let path = map.get("path").and_then(|v| v.as_str()).unwrap_or("/");
            let mut headers = serde_json::Map::new();
            if let Some(h) = map.get("host").and_then(|v| v.as_str()) {
                headers.insert("Host".to_string(), json!(h));
            }
            obj["transport"] = json!({
                "type": "ws",
                "path": path,
                "headers": headers
            });
        } else if net_type == "grpc" {
            let service_name = map.get("service_name")
                .or_else(|| map.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            obj["transport"] = json!({
                "type": "grpc",
                "service_name": service_name
            });
        } else if net_type == "httpupgrade" {
            let path = map.get("path").and_then(|v| v.as_str()).unwrap_or("/");
            let mut trans = json!({
                "type": "httpupgrade",
                "path": path
            });
            if let Some(h) = map.get("host").and_then(|v| v.as_str()) {
                trans["host"] = json!(h);
            }
            obj["transport"] = trans;
        }
    }

    obj
}
