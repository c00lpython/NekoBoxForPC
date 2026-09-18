//! Специализированный интерактивный конструктор балансировщика (Balancer).

use crate::constructor::proxychain::CandidateNode;
use crate::models::ProtocolType;
use crate::storage::StoredProfile;
use serde_json::json;
use std::io::{BufRead, Write};
use uuid::Uuid;

/// Пошаговый интерактивный мастер создания Balancer.
pub fn build_balancer_dialog<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    available_groups: &[String],
    available_nodes: &[CandidateNode],
    profile_name: &str,
    existing: Option<&StoredProfile>,
) -> Result<StoredProfile, String> {
    writeln!(
        writer,
        "\n=== Конструктор балансировщика (Balancer): '{}' ===",
        profile_name
    ).map_err(|e| e.to_string())?;

    writeln!(
        writer,
        "Балансировщик тестирует задержки кандидатов и направляет трафик на оптимальный узел.\n"
    ).map_err(|e| e.to_string())?;

    // Шаг 1: Выбор группы узлов
    writeln!(writer, "Шаг 1/5. Источник узлов для балансировки:").map_err(|e| e.to_string())?;
    for (i, grp) in available_groups.iter().enumerate() {
        let count = available_nodes.iter().filter(|n| &n.group == grp).count();
        writeln!(writer, "  [{}] Группа '{}' ({} узлов)", i + 1, grp, count)
            .map_err(|e| e.to_string())?;
    }
    let all_idx = available_groups.len() + 1;
    writeln!(writer, "  [{}] Все доступные узлы базы ({} узлов)", all_idx, available_nodes.len())
        .map_err(|e| e.to_string())?;

    let default_grp = existing
        .and_then(|e| e.settings.get("group").and_then(|g| g.as_str()))
        .unwrap_or_else(|| {
            if !available_groups.is_empty() {
                &available_groups[0]
            } else {
                "ALL"
            }
        });

    writeln!(
        writer,
        "Выберите группу (номер или имя) [по умолчанию: {}]:",
        default_grp
    ).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut grp_input = String::new();
    reader.read_line(&mut grp_input).map_err(|e| e.to_string())?;
    let trimmed_grp = grp_input.trim();

    let (chosen_group, candidate_tags): (String, Vec<String>) = if trimmed_grp.is_empty() {
        let tags = if default_grp == "ALL" {
            available_nodes.iter().map(|n| n.full_tag()).collect()
        } else {
            available_nodes
                .iter()
                .filter(|n| n.group == default_grp)
                .map(|n| n.full_tag())
                .collect()
        };
        (default_grp.to_string(), tags)
    } else if let Ok(idx) = trimmed_grp.parse::<usize>() {
        if idx >= 1 && idx <= available_groups.len() {
            let grp = &available_groups[idx - 1];
            let tags = available_nodes
                .iter()
                .filter(|n| &n.group == grp)
                .map(|n| n.full_tag())
                .collect();
            (grp.clone(), tags)
        } else {
            // Все узлы
            ("ALL".to_string(), available_nodes.iter().map(|n| n.full_tag()).collect())
        }
    } else {
        let tags = available_nodes
            .iter()
            .filter(|n| n.group.eq_ignore_ascii_case(trimmed_grp))
            .map(|n| n.full_tag())
            .collect();
        (trimmed_grp.to_string(), tags)
    };

    // Шаг 2: Алгоритм балансировки
    let current_strat = existing
        .and_then(|e| e.settings.get("strategy").and_then(|s| s.as_str()))
        .unwrap_or("urltest");

    writeln!(writer, "\nШаг 2/5. Тип проверки / стратегия:").map_err(|e| e.to_string())?;
    writeln!(writer, "  [1] urltest   — автотест задержки и выбор наилучшего пинга").map_err(|e| e.to_string())?;
    writeln!(writer, "  [2] selector  — ручной селектор").map_err(|e| e.to_string())?;
    writeln!(writer, "  [3] fallback  — резервный канал при падении основного").map_err(|e| e.to_string())?;
    writeln!(
        writer,
        "Выберите стратегию [1-3] [текущее: {}]:",
        current_strat
    ).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut strat_input = String::new();
    reader.read_line(&mut strat_input).map_err(|e| e.to_string())?;
    let strategy = match strat_input.trim() {
        "1" => "urltest".to_string(),
        "2" => "selector".to_string(),
        "3" => "fallback".to_string(),
        s if !s.is_empty() => s.to_lowercase(),
        _ => current_strat.to_string(),
    };

    // Шаг 3: Интервал проверки
    let current_interval = existing
        .and_then(|e| e.settings.get("interval").and_then(|i| i.as_str()))
        .unwrap_or("3m");

    writeln!(
        writer,
        "\nШаг 3/5. Интервал повторной проверки задержки (например: 30s, 1m, 3m, 5m) [текущее: {}]:",
        current_interval
    ).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut interval_input = String::new();
    reader.read_line(&mut interval_input).map_err(|e| e.to_string())?;
    let interval = if interval_input.trim().is_empty() {
        current_interval.to_string()
    } else {
        interval_input.trim().to_string()
    };

    // Шаг 4: Допуск задержки (Tolerance)
    let current_tolerance = existing
        .and_then(|e| e.settings.get("tolerance").and_then(|t| t.as_u64()))
        .map(|t| t as u16)
        .unwrap_or(50);

    writeln!(
        writer,
        "\nШаг 4/5. Допуск задержки (Tolerance, ms) — защита от частых переключений [текущее: {}]:",
        current_tolerance
    ).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut tol_input = String::new();
    reader.read_line(&mut tol_input).map_err(|e| e.to_string())?;
    let tolerance = tol_input
        .trim()
        .parse::<u16>()
        .unwrap_or(current_tolerance);

    // Шаг 5: Разрыв соединений (Interrupt Existing Connections)
    let current_interrupt = existing
        .and_then(|e| e.settings.get("interrupt_exist_connections").and_then(|b| b.as_bool()))
        .unwrap_or(true);

    writeln!(
        writer,
        "\nШаг 5/5. Разрывать активные соединения при переключении на более быстрый узел? [Y/n] [текущее: {}]:",
        if current_interrupt { "Y" } else { "N" }
    ).map_err(|e| e.to_string())?;
    write!(writer, "> ").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut int_input = String::new();
    reader.read_line(&mut int_input).map_err(|e| e.to_string())?;
    let interrupt = if int_input.trim().is_empty() {
        current_interrupt
    } else {
        !int_input.trim().eq_ignore_ascii_case("n")
    };

    // Сводка конфигурации
    writeln!(writer, "\n┌─────────────────────────────────────────────────────────────────────────────┐").map_err(|e| e.to_string())?;
    writeln!(writer, "│ Конфигурация Balancer: {:<52} │", profile_name).map_err(|e| e.to_string())?;
    writeln!(writer, "│   • Целевая группа:       {:<49} │", chosen_group).map_err(|e| e.to_string())?;
    writeln!(writer, "│   • Узлов в пуле:         {:<49} │", candidate_tags.len()).map_err(|e| e.to_string())?;
    writeln!(writer, "│   • Стратегия:            {:<49} │", strategy).map_err(|e| e.to_string())?;
    writeln!(writer, "│   • Интервал замера:      {:<49} │", interval).map_err(|e| e.to_string())?;
    writeln!(writer, "│   • Допуск (Tolerance):   {} ms{:<45} │", tolerance, "").map_err(|e| e.to_string())?;
    writeln!(writer, "│   • Разрыв соединений:    {:<49} │", if interrupt { "Включен (true)" } else { "Отключен (false)" }).map_err(|e| e.to_string())?;
    writeln!(writer, "└─────────────────────────────────────────────────────────────────────────────┘\n").map_err(|e| e.to_string())?;

    let id = existing
        .map(|e| e.id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let settings = json!({
        "type": "urltest",
        "tag": profile_name,
        "group": chosen_group,
        "candidates": candidate_tags,
        "strategy": strategy,
        "url": "https://www.gstatic.com/generate_204",
        "interval": interval,
        "tolerance": tolerance,
        "interrupt_exist_connections": interrupt
    });

    Ok(StoredProfile {
        id,
        name: profile_name.to_string(),
        protocol: ProtocolType::Balancer,
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
    fn test_build_balancer_dialog_basic() {
        let groups = vec!["ALLVPN".to_string(), "WARP".to_string()];
        let candidates = vec![
            CandidateNode {
                index: 1,
                group: "ALLVPN".to_string(),
                name: "Node1".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(20),
            },
            CandidateNode {
                index: 2,
                group: "ALLVPN".to_string(),
                name: "Node2".to_string(),
                protocol: "vless".to_string(),
                latency_ms: Some(30),
            },
        ];

        // 1. Выбор группы "1" (ALLVPN)
        // 2. Стратегия Enter (urltest)
        // 3. Интервал Enter (3m)
        // 4. Допуск "60"
        // 5. Разрыв соединений "y"
        let input_data = "1\n\n\n60\ny\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let profile = build_balancer_dialog(
            &mut reader,
            &mut writer,
            &groups,
            &candidates,
            "Best-Auto",
            None,
        ).expect("Balancer dialog failed");

        assert_eq!(profile.name, "Best-Auto");
        assert_eq!(profile.protocol, ProtocolType::Balancer);
        assert_eq!(profile.settings["group"], "ALLVPN");
        assert_eq!(profile.settings["tolerance"], 60);
        assert_eq!(profile.settings["interrupt_exist_connections"], true);
    }
}
