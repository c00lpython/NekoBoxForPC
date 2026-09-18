//! Конструктор групп подписок и ручных коллекций (Instant и Interactive).

use crate::storage::StoredGroup;
use serde_json::Value;
use std::io::{BufRead, Write};

/// Моментальное создание/импорт группы из сырого JSON
pub fn build_group_instant(raw_json: &str) -> Result<StoredGroup, String> {
    let val: Value = serde_json::from_str(raw_json)
        .map_err(|e| format!("Некорректный JSON группы: {}", e))?;

    let name = val
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| "Отсутствует обязательное поле 'name' в описании группы".to_string())?
        .to_string();

    let group_type = val
        .get("group_type")
        .or_else(|| val.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("manual")
        .to_string();

    let subscription_url = val
        .get("subscription_url")
        .or_else(|| val.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    let auto_update_minutes = val
        .get("auto_update_minutes")
        .and_then(|m| m.as_u64())
        .map(|m| m as u32)
        .unwrap_or(if group_type == "sub" { 60 } else { 0 });

    let hwid = val
        .get("hwid")
        .and_then(|h| h.as_bool())
        .unwrap_or(false);

    let client_imitation = val
        .get("client_imitation")
        .and_then(|c| c.as_str())
        .unwrap_or("v2rayN")
        .to_string();

    let front_proxy = val
        .get("front_proxy")
        .and_then(|f| f.as_str())
        .map(|s| s.to_string());

    let outbound_proxy = val
        .get("outbound_proxy")
        .and_then(|o| o.as_str())
        .map(|s| s.to_string());

    Ok(StoredGroup {
        name,
        group_type,
        subscription_url,
        auto_update_minutes,
        hwid,
        client_imitation,
        front_proxy,
        outbound_proxy,
        profiles: Vec::new(),
        subscription_userinfo: None,
        last_updated_at: None,
        error_message: None,
    })
}

/// Пошаговый диалоговый конструктор/редактор группы
pub fn build_group_interactive<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    existing: Option<&StoredGroup>,
) -> Result<StoredGroup, String> {
    writeln!(
        writer,
        "=== {} группы ===",
        if existing.is_some() { "Редактирование" } else { "Конструктор" }
    ).map_err(|e| e.to_string())?;
    writeln!(
        writer,
        "Подсказка: нажмите Enter для выбора дефолта, введите '<' для возврата к предыдущему полю.\n"
    ).map_err(|e| e.to_string())?;

    let mut name = existing
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "MyGroup".to_string());
    let mut group_type = existing
        .map(|e| e.group_type.clone())
        .unwrap_or_else(|| "manual".to_string());
    let mut sub_url = existing
        .and_then(|e| e.subscription_url.clone())
        .unwrap_or_default();
    let mut update_mins = existing
        .map(|e| e.auto_update_minutes)
        .unwrap_or(0);
    let mut imitation = existing
        .map(|e| e.client_imitation.clone())
        .unwrap_or_else(|| "v2rayN".to_string());
    let mut hwid = existing.map(|e| e.hwid).unwrap_or(false);

    let mut step = 0;
    while step <= 5 {
        match step {
            0 => {
                // Name
                writeln!(writer, "Шаг 1/6. Имя группы [текущее: {}]:", name)
                    .map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    continue;
                }
                if !val.is_empty() {
                    name = val.to_string();
                }
                step += 1;
            }
            1 => {
                // Group type: manual / sub
                writeln!(
                    writer,
                    "Шаг 2/6. Тип группы [manual (ручная) / sub (подписка)] [текущее: {}]:",
                    group_type
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim().to_lowercase();
                if val == "<" || val == "prev" {
                    step -= 1;
                    continue;
                }
                if !val.is_empty() {
                    if val == "sub" || val == "subscription" {
                        group_type = "sub".to_string();
                        if update_mins == 0 {
                            update_mins = 60;
                        }
                    } else {
                        group_type = "manual".to_string();
                        update_mins = 0;
                    }
                }
                step += 1;
            }
            2 => {
                // URL подписки (только если sub)
                if group_type == "sub" {
                    writeln!(
                        writer,
                        "Шаг 3/6. URL подписки [текущее: {}]:",
                        if sub_url.is_empty() { "none" } else { &sub_url }
                    ).map_err(|e| e.to_string())?;
                    write!(writer, "> ").map_err(|e| e.to_string())?;
                    writer.flush().map_err(|e| e.to_string())?;

                    let mut input = String::new();
                    reader.read_line(&mut input).map_err(|e| e.to_string())?;
                    let val = input.trim();
                    if val == "<" || val == "prev" {
                        step -= 1;
                        continue;
                    }
                    if !val.is_empty() {
                        sub_url = val.to_string();
                    }
                }
                step += 1;
            }
            3 => {
                // Интервал обновления
                if group_type == "sub" {
                    writeln!(
                        writer,
                        "Шаг 4/6. Интервал обновления в минутах (0 - отключено, 30, 60, 120, 1440) [текущее: {}]:",
                        update_mins
                    ).map_err(|e| e.to_string())?;
                    write!(writer, "> ").map_err(|e| e.to_string())?;
                    writer.flush().map_err(|e| e.to_string())?;

                    let mut input = String::new();
                    reader.read_line(&mut input).map_err(|e| e.to_string())?;
                    let val = input.trim();
                    if val == "<" || val == "prev" {
                        step -= 1;
                        continue;
                    }
                    if !val.is_empty() {
                        if let Ok(m) = val.parse::<u32>() {
                            update_mins = m;
                        }
                    }
                }
                step += 1;
            }
            4 => {
                // Client imitation
                writeln!(
                    writer,
                    "Шаг 5/6. Имитация клиента (User-Agent) [v2rayN, ClashMeta, sing-box] [текущее: {}]:",
                    imitation
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim();
                if val == "<" || val == "prev" {
                    step -= 1;
                    continue;
                }
                if !val.is_empty() {
                    imitation = val.to_string();
                }
                step += 1;
            }
            5 => {
                // HWID
                writeln!(
                    writer,
                    "Шаг 6/6. Отправлять HWID заголовок? [y/N] [текущее: {}]:",
                    if hwid { "y" } else { "n" }
                ).map_err(|e| e.to_string())?;
                write!(writer, "> ").map_err(|e| e.to_string())?;
                writer.flush().map_err(|e| e.to_string())?;

                let mut input = String::new();
                reader.read_line(&mut input).map_err(|e| e.to_string())?;
                let val = input.trim().to_lowercase();
                if val == "<" || val == "prev" {
                    step -= 1;
                    continue;
                }
                if !val.is_empty() {
                    hwid = val == "y" || val == "yes" || val == "true";
                }
                step += 1;
            }
            _ => break,
        }
    }

    let existing_profiles = existing.map(|e| e.profiles.clone()).unwrap_or_default();

    Ok(StoredGroup {
        name,
        group_type,
        subscription_url: if sub_url.is_empty() { None } else { Some(sub_url) },
        auto_update_minutes: update_mins,
        hwid,
        client_imitation: imitation,
        front_proxy: existing.and_then(|e| e.front_proxy.clone()),
        outbound_proxy: existing.and_then(|e| e.outbound_proxy.clone()),
        profiles: existing_profiles,
        subscription_userinfo: existing.and_then(|e| e.subscription_userinfo.clone()),
        last_updated_at: existing.and_then(|e| e.last_updated_at),
        error_message: existing.and_then(|e| e.error_message.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_group_instant() {
        let raw = r#"{
            "name": "VPN-Subs",
            "type": "sub",
            "url": "https://example.com/api/v1/client/subscribe?token=abc",
            "auto_update_minutes": 120,
            "hwid": true
        }"#;

        let g = build_group_instant(raw).expect("Instant group parse failed");
        assert_eq!(g.name, "VPN-Subs");
        assert_eq!(g.group_type, "sub");
        assert_eq!(
            g.subscription_url.as_deref(),
            Some("https://example.com/api/v1/client/subscribe?token=abc")
        );
        assert_eq!(g.auto_update_minutes, 120);
        assert!(g.hwid);
    }

    #[test]
    fn test_build_group_interactive() {
        let input_data = "MyFastGroup\nsub\nhttps://mysub.net/link\n30\nClashMeta\ny\n";
        let mut reader = std::io::Cursor::new(input_data);
        let mut writer = Vec::new();

        let g = build_group_interactive(&mut reader, &mut writer, None)
            .expect("Interactive group build failed");

        assert_eq!(g.name, "MyFastGroup");
        assert_eq!(g.group_type, "sub");
        assert_eq!(g.subscription_url.as_deref(), Some("https://mysub.net/link"));
        assert_eq!(g.auto_update_minutes, 30);
        assert_eq!(g.client_imitation, "ClashMeta");
        assert!(g.hwid);
    }
}
