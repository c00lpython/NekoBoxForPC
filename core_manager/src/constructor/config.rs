//! Конструктор конфигураций профилей (Instant и Interactive).

use crate::models::ProtocolType;
use crate::storage::StoredProfile;
use serde_json::Value;
use std::io::{BufRead, Write};
use uuid::Uuid;

/// Моментальное создание профиля из сырого JSON формата sing-box (outbound или полного конфига)
pub fn build_profile_instant(raw_json: &str) -> Result<StoredProfile, String> {
    let val: Value = serde_json::from_str(raw_json)
        .map_err(|e| format!("Некорректный JSON конфига: {}", e))?;

    // Проверяем, является ли JSON полным sing-box конфигом со списком outbounds
    let outbound_obj = if let Some(outbounds) = val.get("outbounds").and_then(|o| o.as_array()) {
        outbounds
            .iter()
            .find(|o| {
                let typ = o.get("type").and_then(|t| t.as_str()).unwrap_or_default();
                !matches!(typ, "direct" | "block" | "dns" | "selector" | "urltest")
            })
            .ok_or_else(|| "В списке outbounds не найдено валидных прокси-серверов".to_string())?
    } else {
        &val
    };

    let proto_str = outbound_obj
        .get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "Отсутствует обязательное поле 'type'".to_string())?;

    let protocol = ProtocolType::from_str(proto_str);
    let tag = outbound_obj
        .get("tag")
        .and_then(|t| t.as_str())
        .unwrap_or("proxy")
        .to_string();

    let server = outbound_obj
        .get("server")
        .and_then(|s| s.as_str())
        .unwrap_or("127.0.0.1")
        .to_string();

    let server_port = outbound_obj
        .get("server_port")
        .and_then(|p| p.as_u64())
        .map(|p| p as u16)
        .unwrap_or(443);

    let id = Uuid::new_v4().to_string();

    Ok(StoredProfile {
        id,
        name: tag.clone(),
        protocol,
        server,
        server_port,
        settings: outbound_obj.clone(),
        tag,
        last_ping_ms: None,
    })
}

/// Стандартные предложения (suggestions) для различных полей
pub struct FieldSuggestions;

impl FieldSuggestions {
    pub const PROTOCOLS: &'static [&'static str] = &[
        "vless", "vmess", "trojan", "shadowsocks", "hysteria2", "wireguard", "amneziawg", "tuic", "ssh", "byedpi", "masterdnsvpn", "proxychain", "balancer", "direct",
    ];
    pub const PORTS: &'static [u16] = &[443, 80, 8080, 2080, 1080, 8443];
    pub const SECURITY: &'static [&'static str] = &["none", "tls", "reality"];
    pub const NETWORKS: &'static [&'static str] = &["tcp", "ws", "grpc", "http", "quic"];
}

/// Пошаговый диалоговый конструктор/редактор профиля.
/// Поддерживает:
/// - Значения по умолчанию из `existing` профиля
/// - Переход назад по вводу `<` или `prev`
/// - Переход вперед (подтверждение) по нажатию Enter
/// - Подсказки стандартных значений
/// Пошаговый диалоговый конструктор/редактор профиля с поддержкой предзаполненных аргументов CLI.
pub fn build_profile_interactive<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    existing: Option<&StoredProfile>,
) -> Result<StoredProfile, String> {
    build_profile_interactive_full(reader, writer, existing, None, None, &[], &[])
}

/// Полнофункциональный диалоговый конструктор, пропускающий уже переданные аргументы CLI
/// и перенаправляющий на специализированные диалоги для ProxyChain и Balancer.
pub fn build_profile_interactive_full<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    existing: Option<&StoredProfile>,
    name_override: Option<&str>,
    protocol_override: Option<ProtocolType>,
    available_groups: &[String],
    available_nodes: &[crate::constructor::proxychain::CandidateNode],
) -> Result<StoredProfile, String> {
    let initial_name = name_override
        .map(|s| s.to_string())
        .or_else(|| existing.map(|e| e.name.clone()))
        .unwrap_or_else(|| "new-node".to_string());

    let skip_protocol_step = protocol_override.is_some();
    let skip_name_step = name_override.is_some();

    // Если протокол передан явно или уже задан в профиле
    let initial_proto = protocol_override
        .or_else(|| existing.map(|e| e.protocol.clone()));

    // Если протокол сразу определен как Chain (ProxyChain)
    if let Some(ProtocolType::Chain) = initial_proto {
        return crate::constructor::proxychain::build_proxychain_dialog(
            reader,
            writer,
            available_nodes,
            &initial_name,
            existing,
        );
    }

    // Если протокол сразу определен как Balancer
    if let Some(ProtocolType::Balancer) = initial_proto {
        return crate::constructor::balancer::build_balancer_dialog(
            reader,
            writer,
            available_groups,
            available_nodes,
            &initial_name,
            existing,
        );
    }

    writeln!(
        writer,
        "=== {} профиля ===",
        if existing.is_some() { "Редактирование" } else { "Конструктор" }
    ).map_err(|e| e.to_string())?;
    writeln!(
        writer,
        "Подсказка: нажмите Enter для выбора дефолта, введите '<' для возврата к предыдущему полю.\n"
    ).map_err(|e| e.to_string())?;

    // Значения полей
    let mut protocol = initial_proto
        .map(|p| p.to_string())
        .unwrap_or_else(|| "vless".to_string());
    let mut name = initial_name;
    let mut server = existing
        .map(|e| e.server.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let mut port = existing.map(|e| e.server_port).unwrap_or(443);
    let mut credential = existing
        .and_then(|e| {
            e.settings.get("uuid")
                .or_else(|| e.settings.get("password"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut security = existing
        .and_then(|e| {
            if e.settings.get("tls").is_some() {
                if e.settings.get("tls").and_then(|t| t.get("reality")).is_some() {
                    Some("reality".to_string())
                } else {
                    Some("tls".to_string())
                }
            } else {
                Some("none".to_string())
            }
        })
        .unwrap_or_else(|| "tls".to_string());
    let mut sni = existing
        .and_then(|e| {
            e.settings
                .get("tls")
                .and_then(|t| t.get("server_name"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    let mut step = if skip_protocol_step {
        if skip_name_step { 2 } else { 1 }
    } else {
        0
    };

    while step <= 6 {
        match step {
            0 => {
                // Protocol
                writeln!(
                    writer,
                    "Шаг 1/7. Протокол [варианты: {}] [текущее: {}]:",
                    FieldSuggestions::PROTOCOLS.join(", "),
                    protocol
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    continue;
                }
                if !val.is_empty() {
                    protocol = val.to_lowercase();
                }

                // Проверяем, не переключился ли пользователь на ProxyChain или Balancer
                if protocol == "proxychain" || protocol == "chain" {
                    return crate::constructor::proxychain::build_proxychain_dialog(
                        reader,
                        writer,
                        available_nodes,
                        &name,
                        existing,
                    );
                }
                if protocol == "balancer" || protocol == "urltest" {
                    return crate::constructor::balancer::build_balancer_dialog(
                        reader,
                        writer,
                        available_groups,
                        available_nodes,
                        &name,
                        existing,
                    );
                }

                step = if skip_name_step { 2 } else { 1 };
            }
            1 => {
                // Name
                writeln!(writer, "Шаг 2/7. Имя / Tag профиля [текущее: {}]:", name)
                    .map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    if !skip_protocol_step {
                        step = 0;
                    }
                    continue;
                }
                if !val.is_empty() {
                    name = val.to_string();
                }
                step = 2;
            }
            2 => {
                // Server
                let server_prompt = match protocol.as_str() {
                    "byedpi" => "Адрес локального хоста [текущее: 127.0.0.1]",
                    "masterdnsvpn" => "Адрес DNS сервера (IP или домен) [текущее: 77.88.8.8]",
                    _ => "Адрес сервера (IP или домен)",
                };
                writeln!(writer, "Шаг 3/7. {} [текущее: {}]:", server_prompt, server)
                    .map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    step = if !skip_name_step {
                        1
                    } else if !skip_protocol_step {
                        0
                    } else {
                        2
                    };
                    continue;
                }
                if !val.is_empty() {
                    server = val.to_string();
                }
                step = 3;
            }
            3 => {
                // Port
                let default_port = match protocol.as_str() {
                    "byedpi" => 1080,
                    "masterdnsvpn" => 53,
                    "shadowsocks" => 8388,
                    "wireguard" | "amneziawg" => 51820,
                    "ssh" => 22,
                    _ => 443,
                };
                if port == 443 && default_port != 443 {
                    port = default_port;
                }

                let port_suggestions = FieldSuggestions::PORTS
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(
                    writer,
                    "Шаг 4/7. Порт [варианты: {}] [текущее: {}]:",
                    port_suggestions, port
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    step = 2;
                    continue;
                }
                if !val.is_empty() {
                    if let Ok(parsed_port) = val.parse::<u16>() {
                        port = parsed_port;
                    } else {
                        writeln!(writer, "Некорректный номер порта, оставлено: {}", port)
                            .map_err(|e| e.to_string())?;
                    }
                }
                step = 4;
            }
            4 => {
                // Credential (UUID / Password / Key)
                let cred_label = match protocol.as_str() {
                    "vless" | "vmess" | "tuic" => "UUID пользователя",
                    "trojan" | "shadowsocks" | "hysteria2" | "ssh" => "Пароль / Ключ",
                    "wireguard" | "amneziawg" => "Приватный ключ (Private Key)",
                    "byedpi" => "Позиция разделения SNI (split_pos)",
                    "masterdnsvpn" => "DNS сервер резолва",
                    _ => "Идентификатор / Секрет",
                };
                writeln!(
                    writer,
                    "Шаг 5/7. {} [текущее: {}]:",
                    cred_label, credential
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    step = 3;
                    continue;
                }
                if !val.is_empty() {
                    credential = val.to_string();
                }
                step = 5;
            }
            5 => {
                // Security / TLS (только для релевантных протоколов)
                if matches!(protocol.as_str(), "byedpi" | "masterdnsvpn" | "wireguard" | "amneziawg" | "ssh") {
                    security = "none".to_string();
                    step = 6;
                    continue;
                }

                writeln!(
                    writer,
                    "Шаг 6/7. Защита TLS [варианты: {}] [текущее: {}]:",
                    FieldSuggestions::SECURITY.join(", "),
                    security
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    step = 4;
                    continue;
                }
                if !val.is_empty() {
                    security = val.to_lowercase();
                }
                step = 6;
            }
            6 => {
                // SNI
                if security == "none" {
                    sni.clear();
                    step = 7;
                    continue;
                }

                writeln!(
                    writer,
                    "Шаг 7/7. SNI / Server Name [текущее: '{}']:",
                    if sni.is_empty() { "none" } else { &sni }
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    step = 5;
                    continue;
                }
                if !val.is_empty() {
                    sni = if val == "none" { String::new() } else { val.to_string() };
                }
                step = 7;
            }
            _ => break,
        }
    }

    // Собираем sing-box outbound settings
    let mut settings_map = serde_json::Map::new();
    settings_map.insert("type".to_string(), Value::String(protocol.clone()));
    settings_map.insert("tag".to_string(), Value::String(name.clone()));
    settings_map.insert("server".to_string(), Value::String(server.clone()));
    settings_map.insert("server_port".to_string(), Value::Number(port.into()));

    match protocol.as_str() {
        "vless" => {
            settings_map.insert("uuid".to_string(), Value::String(credential));
        }
        "vmess" => {
            settings_map.insert("uuid".to_string(), Value::String(credential));
            settings_map.insert("alter_id".to_string(), Value::Number(0.into()));
        }
        "trojan" | "hysteria2" => {
            settings_map.insert("password".to_string(), Value::String(credential));
        }
        "shadowsocks" => {
            settings_map.insert("password".to_string(), Value::String(credential));
            settings_map.insert("method".to_string(), Value::String("2022-blake3-aes-128-gcm".to_string()));
        }
        "byedpi" | "dpi" => {
            let split_val = credential.parse::<u16>().unwrap_or(2);
            settings_map.insert("split_pos".to_string(), Value::Number(split_val.into()));
            settings_map.insert("split_flags".to_string(), Value::String("--split 2+s".to_string()));
        }
        "masterdnsvpn" | "masterdns" => {
            let dns = if credential.is_empty() { "77.88.8.8".to_string() } else { credential };
            settings_map.insert("dns_server".to_string(), Value::String(dns));
        }
        _ => {
            settings_map.insert("password".to_string(), Value::String(credential));
        }
    }

    if security == "tls" {
        let mut tls_obj = serde_json::Map::new();
        tls_obj.insert("enabled".to_string(), Value::Bool(true));
        if !sni.is_empty() {
            tls_obj.insert("server_name".to_string(), Value::String(sni));
        }
        settings_map.insert("tls".to_string(), Value::Object(tls_obj));
    } else if security == "reality" {
        let mut tls_obj = serde_json::Map::new();
        tls_obj.insert("enabled".to_string(), Value::Bool(true));
        if !sni.is_empty() {
            tls_obj.insert("server_name".to_string(), Value::String(sni));
        }
        let mut reality_obj = serde_json::Map::new();
        reality_obj.insert("enabled".to_string(), Value::Bool(true));
        tls_obj.insert("reality".to_string(), Value::Object(reality_obj));
        settings_map.insert("tls".to_string(), Value::Object(tls_obj));
    }

    let id = existing
        .map(|e| e.id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    Ok(StoredProfile {
        id,
        name: name.clone(),
        protocol: ProtocolType::from_str(&protocol),
        server,
        server_port: port,
        settings: Value::Object(settings_map),
        tag: name,
        last_ping_ms: existing.and_then(|e| e.last_ping_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_profile_instant_raw_outbound() {
        let raw = r#"{
            "type": "vless",
            "tag": "nl-fast",
            "server": "94.130.1.2",
            "server_port": 8443,
            "uuid": "e9a0f02c-5541-4cf1-8848-0d12e6981881"
        }"#;

        let prof = build_profile_instant(raw).expect("Failed to parse instant profile");
        assert_eq!(prof.name, "nl-fast");
        assert_eq!(prof.server, "94.130.1.2");
        assert_eq!(prof.server_port, 8443);
        assert_eq!(prof.protocol, ProtocolType::Vless);
    }

    #[test]
    fn test_build_profile_instant_full_config() {
        let raw = r#"{
            "outbounds": [
                { "type": "direct", "tag": "direct" },
                {
                    "type": "trojan",
                    "tag": "de-trojan",
                    "server": "1.2.3.4",
                    "server_port": 443,
                    "password": "secretpassword"
                }
            ]
        }"#;

        let prof = build_profile_instant(raw).expect("Failed to extract proxy outbound");
        assert_eq!(prof.name, "de-trojan");
        assert_eq!(prof.protocol, ProtocolType::Trojan);
        assert_eq!(prof.server_port, 443);
    }

    #[test]
    fn test_build_profile_interactive_with_prev_navigation() {
        // Симулируем ввод:
        // 1. "vmess"
        // 2. "my-test"
        // 3. "<" (возврат к имени)
        // 4. "my-test-v2"
        // 5. "10.0.0.1"
        // 6. Enter (дефолтный порт 443)
        // 7. "my-uuid-key"
        // 8. "tls"
        // 9. "example.com"
        let input_data = "vmess\nmy-test\n<\nmy-test-v2\n10.0.0.1\n\nmy-uuid-key\ntls\nexample.com\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let prof = build_profile_interactive(&mut reader, &mut writer, None)
            .expect("Interactive wizard failed");

        assert_eq!(prof.protocol, ProtocolType::Vmess);
        assert_eq!(prof.name, "my-test-v2");
        assert_eq!(prof.server, "10.0.0.1");
        assert_eq!(prof.server_port, 443);
        assert_eq!(prof.settings["uuid"], "my-uuid-key");
        assert_eq!(prof.settings["tls"]["server_name"], "example.com");
    }

    #[test]
    fn test_build_profile_interactive_skip_passed_args() {
        // Заданы name_override = "Predefined-Node" и protocol_override = Vless
        // Конструктор НЕ должен спрашивать протокол и имя! Сразу запрашивает:
        // 1. Сервер: "1.2.3.4"
        // 2. Порт: Enter (443)
        // 3. UUID: "my-custom-uuid"
        // 4. Security: "tls"
        // 5. SNI: "fast.example.com"
        let input_data = "1.2.3.4\n\nmy-custom-uuid\ntls\nfast.example.com\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let prof = build_profile_interactive_full(
            &mut reader,
            &mut writer,
            None,
            Some("Predefined-Node"),
            Some(ProtocolType::Vless),
            &[],
            &[],
        ).expect("Failed with passed args");

        assert_eq!(prof.name, "Predefined-Node");
        assert_eq!(prof.protocol, ProtocolType::Vless);
        assert_eq!(prof.server, "1.2.3.4");
        assert_eq!(prof.server_port, 443);
        assert_eq!(prof.settings["uuid"], "my-custom-uuid");
        assert_eq!(prof.settings["tls"]["server_name"], "fast.example.com");
    }

    #[test]
    fn test_build_profile_interactive_delegates_to_proxychain() {
        let candidates = vec![
            crate::constructor::proxychain::CandidateNode {
                index: 1,
                group: "GRP".to_string(),
                name: "NodeA".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(10),
            },
            crate::constructor::proxychain::CandidateNode {
                index: 2,
                group: "GRP".to_string(),
                name: "NodeB".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(20),
            },
        ];

        let input_data = "1\n2\n\ny\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let prof = build_profile_interactive_full(
            &mut reader,
            &mut writer,
            None,
            Some("AutoChain"),
            Some(ProtocolType::Chain),
            &["GRP".to_string()],
            &candidates,
        ).expect("Chain delegation failed");

        assert_eq!(prof.name, "AutoChain");
        assert_eq!(prof.protocol, ProtocolType::Chain);
        let nodes = prof.settings["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
    }
}

