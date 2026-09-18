//! Специализированный интерактивный конструктор цепочек прокси (ProxyChain).

use crate::models::ProtocolType;
use crate::storage::StoredProfile;
use serde_json::json;
use std::io::{BufRead, Write};
use uuid::Uuid;

/// Информация о доступном узле для выбора в цепочку
#[derive(Debug, Clone)]
pub struct CandidateNode {
    pub index: usize,
    pub group: String,
    pub name: String,
    pub protocol: String,
    pub latency_ms: Option<u64>,
}

impl CandidateNode {
    pub fn full_tag(&self) -> String {
        format!("{}.{}", self.group, self.name)
    }
}

/// Пошаговый интерактивный мастер создания ProxyChain.
pub fn build_proxychain_dialog<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    available_nodes: &[CandidateNode],
    profile_name: &str,
    existing: Option<&StoredProfile>,
) -> Result<StoredProfile, String> {
    writeln!(
        writer,
        "\n=== Конструктор цепочки прокси (ProxyChain): '{}' ===",
        profile_name
    ).map_err(|e| e.to_string())?;

    writeln!(
        writer,
        "Трафик пойдет последовательно: Клиент -> Хоп 1 -> Хоп 2 -> ... -> Интернет\n"
    ).map_err(|e| e.to_string())?;

    let mut selected_node_tags: Vec<String> = Vec::new();

    if !available_nodes.is_empty() {
        writeln!(writer, "Доступные узлы в хранилище:").map_err(|e| e.to_string())?;
        for node in available_nodes {
            let ping_str = match node.latency_ms {
                Some(ms) => format!("{:>4} ms", ms),
                None => " --- ms".to_string(),
            };
            writeln!(
                writer,
                "  [{:>2}] ({:<10}) {:<30} | {:<10} | {}",
                node.index, node.group, node.name, node.protocol, ping_str
            ).map_err(|e| e.to_string())?;
        }
        writeln!(writer).map_err(|e| e.to_string())?;
    }

    let existing_nodes: Vec<String> = existing
        .and_then(|e| e.settings.get("nodes").and_then(|n| n.as_array()))
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Хоп 1: Входной узел
    let default_hop1 = existing_nodes.first().cloned();
    let hop1_prompt = if let Some(ref d) = default_hop1 {
        format!("Хоп 1 [Входной узел (Entry node)]: укажите номер или имя узла [текущее: {}]:", d)
    } else {
        "Хоп 1 [Входной узел (Entry node)]: укажите номер или имя узла:".to_string()
    };
    writeln!(writer, "{}", hop1_prompt).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut input1 = String::new();
    reader.read_line(&mut input1).map_err(|e| e.to_string())?;
    let val1 = input1.trim();
    let hop1 = if val1.is_empty() {
        default_hop1.unwrap_or_else(|| {
            available_nodes.first().map(|n| n.full_tag()).unwrap_or_else(|| "node-1".to_string())
        })
    } else if let Ok(idx) = val1.parse::<usize>() {
        available_nodes.iter().find(|n| n.index == idx).map(|n| n.full_tag()).unwrap_or_else(|| val1.to_string())
    } else {
        val1.to_string()
    };
    selected_node_tags.push(hop1);

    // Хоп 2: Выходной узел
    let default_hop2 = existing_nodes.get(1).cloned();
    let hop2_prompt = if let Some(ref d) = default_hop2 {
        format!("Хоп 2 [Выходной узел (Exit node)]: укажите номер или имя узла [текущее: {}]:", d)
    } else {
        "Хоп 2 [Выходной узел (Exit node)]: укажите номер или имя узла:".to_string()
    };
    writeln!(writer, "{}", hop2_prompt).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut input2 = String::new();
    reader.read_line(&mut input2).map_err(|e| e.to_string())?;
    let val2 = input2.trim();
    let hop2 = if val2.is_empty() {
        default_hop2.unwrap_or_else(|| {
            if available_nodes.len() > 1 {
                available_nodes[1].full_tag()
            } else {
                available_nodes.first().map(|n| n.full_tag()).unwrap_or_else(|| "node-2".to_string())
            }
        })
    } else if let Ok(idx) = val2.parse::<usize>() {
        available_nodes.iter().find(|n| n.index == idx).map(|n| n.full_tag()).unwrap_or_else(|| val2.to_string())
    } else {
        val2.to_string()
    };
    selected_node_tags.push(hop2);

    // Опция добавления дополнительных узлов в цепочку (Хоп 3, 4, ...)
    let mut hop_idx = 3;
    loop {
        let default_hop_n = existing_nodes.get(hop_idx - 1).cloned();
        let prompt = if let Some(ref d) = default_hop_n {
            format!("Добавить Хоп {}? (номер/имя узла или Enter для сохранения [текущее: {}]):", hop_idx, d)
        } else {
            format!("Добавить Хоп {}? (номер/имя узла или Enter для завершения выбора):", hop_idx)
        };
        writeln!(writer, "{}", prompt).map_err(|e| e.to_string())?;
        write!(writer, "> ").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;

        let mut next_input = String::new();
        reader.read_line(&mut next_input).map_err(|e| e.to_string())?;
        let v = next_input.trim();
        if v.is_empty() {
            if let Some(d) = default_hop_n {
                selected_node_tags.push(d);
                hop_idx += 1;
                continue;
            } else {
                break;
            }
        }
        if v.eq_ignore_ascii_case("done") || v.eq_ignore_ascii_case("no") || v.eq_ignore_ascii_case("n") {
            break;
        }
        let hop_tag = if let Ok(idx) = v.parse::<usize>() {
            available_nodes.iter().find(|n| n.index == idx).map(|n| n.full_tag()).unwrap_or_else(|| v.to_string())
        } else {
            v.to_string()
        };
        selected_node_tags.push(hop_tag);
        hop_idx += 1;
    }

    // UDP опция
    let default_udp = existing
        .and_then(|e| e.settings.get("udp").and_then(|u| u.as_bool()))
        .unwrap_or(true);

    writeln!(
        writer,
        "Проксировать UDP-трафик через цепочку? [Y/n] [текущее: {}]:",
        if default_udp { "Y" } else { "n" }
    ).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut udp_input = String::new();
    reader.read_line(&mut udp_input).map_err(|e| e.to_string())?;
    let udp_val = udp_input.trim();
    let udp_enabled = if udp_val.is_empty() {
        default_udp
    } else {
        !udp_val.eq_ignore_ascii_case("n")
    };

    // Отрисовка схемы топологии
    writeln!(writer, "\n┌─────────────────────────────────────────────────────────────────────────────┐").map_err(|e| e.to_string())?;
    writeln!(writer, "│ Топология ProxyChain:                                                       │").map_err(|e| e.to_string())?;
    let n = selected_node_tags.len();
    for (i, tag) in selected_node_tags.iter().enumerate() {
        let label = if i == 0 {
            "[Хоп 1: Вход]"
        } else if i == n - 1 {
            "[Хоп N: Выход в Интернет]"
        } else {
            "[Хоп промежуточный]"
        };
        writeln!(writer, "│   {:<25} -> {:<45} │", label, tag).map_err(|e| e.to_string())?;
    }
    writeln!(writer, "│ UDP поддержка: {:<60} │", if udp_enabled { "Включена" } else { "Отключена" }).map_err(|e| e.to_string())?;
    writeln!(writer, "└─────────────────────────────────────────────────────────────────────────────┘\n").map_err(|e| e.to_string())?;

    let id = existing
        .map(|e| e.id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let settings = json!({
        "type": "chain",
        "tag": profile_name,
        "nodes": selected_node_tags,
        "udp": udp_enabled
    });

    Ok(StoredProfile {
        id,
        name: profile_name.to_string(),
        protocol: ProtocolType::Chain,
        server: "127.0.0.1".to_string(),
        server_port: 0,
        settings,
        tag: profile_name.to_string(),
        last_ping_ms: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_proxychain_dialog_step_by_step() {
        let candidates = vec![
            CandidateNode {
                index: 1,
                group: "ALLVPN".to_string(),
                name: "Node-RU".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(60),
            },
            CandidateNode {
                index: 2,
                group: "ALLVPN".to_string(),
                name: "Node-DE".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(40),
            },
            CandidateNode {
                index: 3,
                group: "WARP".to_string(),
                name: "Cloudflare".to_string(),
                protocol: "wireguard".to_string(),
                latency_ms: Some(2),
            },
        ];

        // Хоп 1: "1", Хоп 2: "3", Добавить Хоп 3: Enter (завершить), UDP: "y"
        let input_data = "1\n3\n\ny\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let profile = build_proxychain_dialog(
            &mut reader,
            &mut writer,
            &candidates,
            "MyChainTest",
            None,
        ).expect("ProxyChain dialog failed");

        assert_eq!(profile.name, "MyChainTest");
        assert_eq!(profile.protocol, ProtocolType::Chain);
        let nodes = profile.settings["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], "ALLVPN.Node-RU");
        assert_eq!(nodes[1], "WARP.Cloudflare");
        assert_eq!(profile.settings["udp"], true);
    }

    #[test]
    fn test_build_proxychain_dialog_edit_existing() {
        let candidates = vec![
            CandidateNode {
                index: 1,
                group: "ALLVPN".to_string(),
                name: "Node-RU".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(60),
            },
            CandidateNode {
                index: 2,
                group: "WARP".to_string(),
                name: "Cloudflare".to_string(),
                protocol: "wireguard".to_string(),
                latency_ms: Some(2),
            },
        ];

        let existing = StoredProfile {
            id: "chain-1".to_string(),
            name: "ExistingChain".to_string(),
            protocol: ProtocolType::Chain,
            server: "127.0.0.1".to_string(),
            server_port: 0,
            settings: json!({
                "type": "chain",
                "nodes": ["ALLVPN.Node-RU", "WARP.Cloudflare"],
                "udp": true
            }),
            tag: "ExistingChain".to_string(),
            last_ping_ms: None,
        };

        // Нажимаем Enter на Хоп 1 (сохранить Node-RU), Enter на Хоп 2 (сохранить Cloudflare),
        // Enter на Хоп 3 (завершить), Enter на UDP (сохранить true)
        let input_data = "\n\n\n\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let profile = build_proxychain_dialog(
            &mut reader,
            &mut writer,
            &candidates,
            "ExistingChain",
            Some(&existing),
        ).expect("ProxyChain dialog failed");

        assert_eq!(profile.id, "chain-1");
        let nodes = profile.settings["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], "ALLVPN.Node-RU");
        assert_eq!(nodes[1], "WARP.Cloudflare");
        assert_eq!(profile.settings["udp"], true);
    }
}
