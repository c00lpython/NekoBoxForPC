//! Комплексные интеграционные тесты для конструктора, селектора и хранилища.

use core_manager::{
    build_group_instant, build_group_interactive, build_profile_instant,
    build_profile_interactive, ConfigStore, ParsedSelector, ProtocolType, StoredGroup,
    StoredProfile,
};
use std::io::Cursor;
use uuid::Uuid;

#[test]
fn test_config_store_crud_and_pick_flow() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_store_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");

    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));
    assert_eq!(store.data.groups.len(), 1);
    assert_eq!(store.data.groups[0].name, "Default");

    // 1. Добавление новой группы
    let mut vpn_group = StoredGroup::new_subscription("FastVPN", "https://api.fastvpn.com/sub");
    vpn_group.auto_update_minutes = 30;
    store.add_group(vpn_group).expect("Failed to add group");
    assert_eq!(store.data.groups.len(), 2);

    // 2. Добавление профилей
    let p1 = StoredProfile {
        id: "p1".to_string(),
        name: "amsterdam-1".to_string(),
        protocol: ProtocolType::Vless,
        server: "1.2.3.4".to_string(),
        server_port: 443,
        settings: serde_json::json!({ "type": "vless" }),
        tag: "amsterdam-1".to_string(),
        last_ping_ms: Some(35),
    };
    let p2 = StoredProfile {
        id: "p2".to_string(),
        name: "berlin-2".to_string(),
        protocol: ProtocolType::Trojan,
        server: "5.6.7.8".to_string(),
        server_port: 8443,
        settings: serde_json::json!({ "type": "trojan" }),
        tag: "berlin-2".to_string(),
        last_ping_ms: Some(48),
    };

    store.add_profile("FastVPN", p1).expect("Failed to add p1");
    store.add_profile("FastVPN", p2).expect("Failed to add p2");

    // 3. Проверка find_profile
    let found = store.find_profile("FastVPN.amsterdam-1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().1.name, "amsterdam-1");

    // 4. Проверка pick и resolve_run_target
    store.pick("FastVPN.berlin-2").expect("Failed to pick");
    assert_eq!(store.data.active_picked.as_deref(), Some("FastVPN.berlin-2"));

    let resolved = store.resolve_run_target(None).expect("Should resolve picked target");
    assert_eq!(resolved.1.name, "berlin-2");
    assert_eq!(store.data.last_run.as_deref(), Some("FastVPN.berlin-2"));

    // 5. Unpick и fallback на last_run
    store.unpick().expect("Failed to unpick");
    assert!(store.data.active_picked.is_none());

    let resolved_last = store.resolve_run_target(None).expect("Should resolve last_run target");
    assert_eq!(resolved_last.1.name, "berlin-2");

    // 6. Удаление профиля
    store.remove_profile("FastVPN.berlin-2").expect("Failed to remove profile");
    assert!(store.find_profile("FastVPN.berlin-2").is_none());

    // 7. Перезагрузка с диска
    let reloaded = ConfigStore::load_or_default(Some(store_file));
    assert_eq!(reloaded.data.groups.len(), 2);
    let reloaded_vpn = reloaded.get_group("FastVPN").expect("Group should persist");
    assert_eq!(reloaded_vpn.profiles.len(), 1);
    assert_eq!(reloaded_vpn.profiles[0].name, "amsterdam-1");

    // Очистка
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_selector_complex_ranges_and_inversion() {
    let mut store = ConfigStore::load_or_default(None);
    store.data.groups.clear();

    let mut g = StoredGroup::new_manual("Servers");
    let names = vec!["alfa", "bravo", "delta", "echo", "foxtrot", "hotel", "india", "zulu"];
    for n in names {
        g.profiles.push(StoredProfile {
            id: n.to_string(),
            name: n.to_string(),
            protocol: ProtocolType::Vless,
            server: "1.1.1.1".to_string(),
            server_port: 443,
            settings: serde_json::json!({}),
            tag: n.to_string(),
            last_ping_ms: None,
        });
    }
    store.data.groups.push(g);

    // Диапазоны [a-c],[h-z]
    // Ожидаем: alfa, bravo, hotel, india, zulu
    // Исключаются: delta, echo, foxtrot
    let selector = ParsedSelector::parse("[a-c],[h-z]");
    let res = selector.select(&store);
    let matched: Vec<String> = res.into_iter().map(|(_, p)| p.name).collect();
    assert_eq!(matched, vec!["alfa", "bravo", "hotel", "india", "zulu"]);

    // Инверсия: invert [a-c],[h-z]
    // Ожидаем: delta, echo, foxtrot
    let inv_selector = ParsedSelector::parse("invert [a-c],[h-z]");
    let inv_res = inv_selector.select(&store);
    let inv_matched: Vec<String> = inv_res.into_iter().map(|(_, p)| p.name).collect();
    assert_eq!(inv_matched, vec!["delta", "echo", "foxtrot"]);
}

#[test]
fn test_constructor_instant_and_interactive() {
    // 1. Instant outbound
    let json_raw = r#"{
        "type": "shadowsocks",
        "tag": "ss-japan",
        "server": "133.1.2.3",
        "server_port": 8388,
        "method": "2022-blake3-aes-128-gcm",
        "password": "mypassword"
    }"#;
    let prof = build_profile_instant(json_raw).expect("Instant profile build failed");
    assert_eq!(prof.name, "ss-japan");
    assert_eq!(prof.protocol, ProtocolType::Shadowsocks);
    assert_eq!(prof.server_port, 8388);

    // 2. Interactive с навигацией назад prev (<)
    // Шаги:
    // 1. vless
    // 2. test-temp
    // 3. < (возврат к имени)
    // 4. real-node
    // 5. 127.0.0.1
    // 6. Enter (дефолтный порт 443)
    // 7. my-uuid-1234
    // 8. reality
    // 9. gateway.example.org
    let input = "vless\ntest-temp\n<\nreal-node\n127.0.0.1\n\nmy-uuid-1234\nreality\ngateway.example.org\n";
    let mut reader = Cursor::new(input);
    let mut writer = Vec::new();

    let interactive_prof = build_profile_interactive(&mut reader, &mut writer, None)
        .expect("Interactive constructor failed");

    assert_eq!(interactive_prof.name, "real-node");
    assert_eq!(interactive_prof.protocol, ProtocolType::Vless);
    assert_eq!(interactive_prof.server, "127.0.0.1");
    assert_eq!(interactive_prof.server_port, 443);
    assert_eq!(interactive_prof.settings["tls"]["reality"]["enabled"], true);
    assert_eq!(interactive_prof.settings["tls"]["server_name"], "gateway.example.org");

    // 3. Instant Group
    let group_json = r#"{
        "name": "AutoUpdateGroup",
        "type": "sub",
        "url": "https://subscribe.net/feed",
        "auto_update_minutes": 15
    }"#;
    let grp = build_group_instant(group_json).expect("Instant group parse failed");
    assert_eq!(grp.name, "AutoUpdateGroup");
    assert_eq!(grp.group_type, "sub");
    assert_eq!(grp.auto_update_minutes, 15);

    // 4. Interactive Group
    let group_input = "InteractiveSub\nsub\nhttps://example.com/sub\n45\nClashMeta\ny\n";
    let mut grp_reader = Cursor::new(group_input);
    let mut grp_writer = Vec::new();
    let interactive_grp = build_group_interactive(&mut grp_reader, &mut grp_writer, None)
        .expect("Interactive group failed");
    assert_eq!(interactive_grp.name, "InteractiveSub");
    assert_eq!(interactive_grp.group_type, "sub");
    assert_eq!(interactive_grp.auto_update_minutes, 45);
    assert_eq!(interactive_grp.client_imitation, "ClashMeta");
    assert!(interactive_grp.hwid);
}

#[test]
fn test_geo_storage_crud() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_geo_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");

    let mut store = ConfigStore::load_or_default(Some(store_file));
    assert!(store.list_geo_files().is_empty());

    // 1. Сохранение файла
    let fake_data = b"GEOSITE_DATABASE_TEST_PAYLOAD";
    let geo = store
        .save_geo_bytes("geosite.db", "geosite", Some("https://example.com/geosite.db".to_string()), fake_data)
        .expect("Failed to save geo bytes");

    assert_eq!(geo.name, "geosite.db");
    assert_eq!(geo.file_type, "geosite");
    assert_eq!(geo.size_bytes, fake_data.len() as u64);
    assert!(geo.sha256.is_some());
    assert_eq!(store.list_geo_files().len(), 1);

    // 2. Проверка существования на диске
    let disk_path = store.get_geo_dir().join("geosite.db");
    assert!(disk_path.exists());
    let read_data = std::fs::read(&disk_path).expect("Failed to read geo file");
    assert_eq!(read_data, fake_data);

    // 3. Удаление
    store.remove_geo_file("geosite.db").expect("Failed to remove geo file");
    assert!(store.list_geo_files().is_empty());
    assert!(!disk_path.exists());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_backup_and_recovery_selective() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_backup_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");
    let backups_dir = temp_dir.join("backups");

    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));

    // Добавляем тестовую группу и правило
    let mut g = StoredGroup::new_manual("BackupTestGroup");
    g.profiles.push(StoredProfile {
        id: "id-1".to_string(),
        name: "node-backup".to_string(),
        protocol: ProtocolType::Vless,
        server: "10.0.0.1".to_string(),
        server_port: 443,
        settings: serde_json::json!({}),
        tag: "node-backup".to_string(),
        last_ping_ms: None,
    });
    store.add_group(g).expect("Failed to add group");
    store.add_rule("RouteBackup", "direct", vec!["test.com".to_string()], vec!["1.1.1.1/32".to_string()]).expect("Failed to add rule");

    // 1. Создание бэкапа (только routes)
    let routes_backup = store
        .create_backup("OnlyRoutes", &backups_dir, false, true, false)
        .expect("Failed to create routes backup");
    assert!(routes_backup.exists());

    // 2. Создание полного бэкапа
    let full_backup = store
        .create_backup("Full", &backups_dir, true, true, true)
        .expect("Failed to create full backup");
    assert!(full_backup.exists());

    // 3. Модификация данных в хранилище (удаление правила и группы)
    store.del_rule("RouteBackup").expect("Failed to del rule");
    store.remove_group("BackupTestGroup").expect("Failed to remove group");
    assert!(store.data.rules.is_empty());
    assert_eq!(store.data.groups.len(), 1); // только Default

    // 4. Восстановление только правил из routes_backup
    let report = store
        .restore_backup(&routes_backup, false, true, false, core_manager::storage::RecoveryMode::Hard)
        .expect("Failed to restore routes");
    assert!(report.routes_restored);
    assert_eq!(report.rules_count, 1);
    assert_eq!(store.data.rules.len(), 1);
    assert_eq!(store.data.rules[0].name, "RouteBackup");
    assert_eq!(store.data.groups.len(), 1); // группы не восстанавливались

    // 5. Полное восстановление из full_backup
    let full_rep = store
        .restore_backup(&full_backup, true, true, true, core_manager::storage::RecoveryMode::Hard)
        .expect("Failed to restore full");
    assert!(full_rep.configs_restored);
    assert_eq!(full_rep.groups_count, 2);
    assert_eq!(store.data.groups.len(), 2);
    assert!(store.get_group("BackupTestGroup").is_some());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_update_pings_batch_flow() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_pings_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");

    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));

    let mut grp = StoredGroup::new_manual("PingBatchTest");
    grp.profiles.push(StoredProfile {
        id: "node1".to_string(),
        name: "server-alpha".to_string(),
        protocol: ProtocolType::Vless,
        server: "1.1.1.1".to_string(),
        server_port: 443,
        settings: serde_json::json!({ "type": "vless" }),
        tag: "server-alpha".to_string(),
        last_ping_ms: None,
    });
    grp.profiles.push(StoredProfile {
        id: "node2".to_string(),
        name: "server-beta".to_string(),
        protocol: ProtocolType::Trojan,
        server: "2.2.2.2".to_string(),
        server_port: 443,
        settings: serde_json::json!({ "type": "trojan" }),
        tag: "server-beta".to_string(),
        last_ping_ms: None,
    });
    store.add_group(grp).expect("Failed to add group");

    // Пакетное обновление
    let updates = vec![
        ("PingBatchTest".to_string(), "server-alpha".to_string(), Some(120)),
        ("PingBatchTest".to_string(), "server-beta".to_string(), Some(45)),
    ];

    let count = store.update_pings_batch(&updates).expect("Batch update failed");
    assert_eq!(count, 2);

    // Проверяем обновленные значения в памяти
    let grp_check = store.get_group("PingBatchTest").unwrap();
    assert_eq!(grp_check.profiles[0].last_ping_ms, Some(120));
    assert_eq!(grp_check.profiles[1].last_ping_ms, Some(45));

    // Проверяем сохранение на диск
    let store_reloaded = ConfigStore::load_or_default(Some(store_file));
    let grp_reloaded = store_reloaded.get_group("PingBatchTest").unwrap();
    assert_eq!(grp_reloaded.profiles[0].last_ping_ms, Some(120));
    assert_eq!(grp_reloaded.profiles[1].last_ping_ms, Some(45));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_ping_sorting_criteria() {
    let items: Vec<(&str, &str, Option<u64>)> = vec![
        ("G1", "slow-node", Some(500)),
        ("G2", "fast-node", Some(30)),
        ("G1", "timeout-node", None),
        ("G2", "mid-node", Some(150)),
    ];

    // 1. Fastest (asc)
    let mut fastest = items.clone();
    fastest.sort_by(|a, b| match (a.2, b.2) {
        (Some(m1), Some(m2)) => m1.cmp(&m2),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.1.cmp(&b.1),
    });
    assert_eq!(fastest[0].1, "fast-node");
    assert_eq!(fastest[1].1, "mid-node");
    assert_eq!(fastest[2].1, "slow-node");
    assert_eq!(fastest[3].1, "timeout-node");

    // 2. Slowest (desc)
    let mut slowest = items.clone();
    slowest.sort_by(|a, b| match (a.2, b.2) {
        (Some(m1), Some(m2)) => m2.cmp(&m1),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.1.cmp(&b.1),
    });
    assert_eq!(slowest[0].1, "slow-node");
    assert_eq!(slowest[1].1, "mid-node");
    assert_eq!(slowest[2].1, "fast-node");
    assert_eq!(slowest[3].1, "timeout-node");

    // 3. Name (alpha)
    let mut by_name = items.clone();
    by_name.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    assert_eq!(by_name[0].1, "fast-node");
    assert_eq!(by_name[1].1, "mid-node");
    assert_eq!(by_name[2].1, "slow-node");
    assert_eq!(by_name[3].1, "timeout-node");

    // 4. Group
    let mut by_group = items.clone();
    by_group.sort_by(|a, b| {
        let g = a.0.cmp(&b.0);
        if g == std::cmp::Ordering::Equal {
            a.1.cmp(&b.1)
        } else {
            g
        }
    });
    assert_eq!(by_group[0].0, "G1");
    assert_eq!(by_group[0].1, "slow-node");
    assert_eq!(by_group[1].0, "G1");
    assert_eq!(by_group[1].1, "timeout-node");
    assert_eq!(by_group[2].0, "G2");
    assert_eq!(by_group[2].1, "fast-node");
    assert_eq!(by_group[3].0, "G2");
    assert_eq!(by_group[3].1, "mid-node");
}

#[test]
fn test_recovery_merge_mode_and_preview() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_merge_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");
    let backups_dir = temp_dir.join("backups");
    std::fs::create_dir_all(&backups_dir).unwrap();

    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));

    // 1. Создаем начальное состояние
    let mut g1 = StoredGroup::new_manual("WorkGroup");
    g1.profiles.push(StoredProfile {
        id: "p1".to_string(),
        name: "node-1".to_string(),
        protocol: ProtocolType::Vless,
        server: "1.1.1.1".to_string(),
        server_port: 443,
        settings: serde_json::json!({}),
        tag: "node-1".to_string(),
        last_ping_ms: Some(100),
    });
    store.add_group(g1).unwrap();
    store.add_rule("RuleA", "direct", vec!["work.com".to_string()], vec![]).unwrap();

    // Создаем бэкап состояния 1
    let backup_path = store.create_backup("BackupV1", &backups_dir, true, true, true).unwrap();

    // 2. Добавляем в текущее хранилище новые элементы (которые не должны быть стерты при merge)
    let mut g2 = StoredGroup::new_manual("HomeGroup");
    g2.profiles.push(StoredProfile {
        id: "p_home".to_string(),
        name: "home-vpn".to_string(),
        protocol: ProtocolType::Trojan,
        server: "9.9.9.9".to_string(),
        server_port: 8443,
        settings: serde_json::json!({}),
        tag: "home-vpn".to_string(),
        last_ping_ms: None,
    });
    store.add_group(g2).unwrap();
    store.add_rule("RuleB", "proxy", vec!["home.com".to_string()], vec![]).unwrap();

    // Также добавим в WorkGroup второй профиль
    let g1_mut = store.get_group_mut("WorkGroup").unwrap();
    g1_mut.profiles.push(StoredProfile {
        id: "p2_local".to_string(),
        name: "node-2-local".to_string(),
        protocol: ProtocolType::Vless,
        server: "2.2.2.2".to_string(),
        server_port: 443,
        settings: serde_json::json!({}),
        tag: "node-2-local".to_string(),
        last_ping_ms: None,
    });
    store.save().unwrap();

    assert_eq!(store.data.groups.len(), 3); // Default, WorkGroup, HomeGroup
    assert_eq!(store.data.rules.len(), 2);  // RuleA, RuleB

    // 3. Проверяем inspect_backup (preview)
    let preview = store.inspect_backup(&backup_path, true, true, true, core_manager::storage::RecoveryMode::Merge).unwrap();
    assert_eq!(preview.backup_name, "BackupV1");
    assert_eq!(preview.groups.len(), 2); // Default, WorkGroup
    assert_eq!(preview.rules.len(), 1);  // RuleA

    // 4. Применяем восстановление в режиме MERGE
    let rep = store.restore_backup(&backup_path, true, true, true, core_manager::storage::RecoveryMode::Merge).unwrap();
    assert!(rep.configs_restored);
    assert!(rep.routes_restored);

    // В режиме MERGE:
    // - HomeGroup ДОЛЖНА сохраниться!
    assert!(store.get_group("HomeGroup").is_some());
    // - WorkGroup должна содержать и старый node-1, и добавленный локально node-2-local!
    let work_g = store.get_group("WorkGroup").unwrap();
    assert_eq!(work_g.profiles.len(), 2);
    assert!(work_g.profiles.iter().any(|p| p.name == "node-1"));
    assert!(work_g.profiles.iter().any(|p| p.name == "node-2-local"));

    // - RuleB ДОЛЖНО сохраниться!
    assert_eq!(store.data.rules.len(), 2);
    assert!(store.data.rules.iter().any(|r| r.name == "RuleA"));
    assert!(store.data.rules.iter().any(|r| r.name == "RuleB"));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_builtin_assets_installation() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_assets_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");

    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));

    // Устанавливаем встроенные ассеты
    let res = store.install_builtin_assets().expect("Failed to install builtin assets");
    assert_eq!(res.len(), 2);

    // Проверяем наличие группы WARP
    let warp_g = store.get_group("WARP").expect("WARP group must exist");
    assert_eq!(warp_g.profiles.len(), 2);
    assert!(warp_g.profiles.iter().any(|p| p.name.contains("AWG")));
    assert!(warp_g.profiles.iter().any(|p| p.name.contains("MASQUE")));

    // Проверяем наличие группы Goida Group
    let goida_g = store.get_group("Goida Group").expect("Goida Group must exist");
    assert_eq!(goida_g.group_type, "sub");
    assert!(goida_g.subscription_url.is_some());
    assert_eq!(goida_g.auto_update_minutes, 720);
    assert!(goida_g.profiles.len() >= 2);

    // Повторный вызов install_builtin_assets не должен дублировать профили
    let _ = store.install_builtin_assets().unwrap();
    let warp_g2 = store.get_group("WARP").unwrap();
    assert_eq!(warp_g2.profiles.len(), 2);

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_income_and_routing_settings_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_test_income_routing_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");

    // 1. Дефолтные настройки
    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));
    let s = &store.data.settings;
    assert!(s.income_local_proxy_in_vpn);
    assert!(!s.income_turn_off_udp);
    assert_eq!(s.income_local_ip, "127.0.0.1");
    assert_eq!(s.income_local_port, 20808);
    assert_eq!(s.income_username, "");
    assert_eq!(s.income_password, "");
    assert!(s.income_http_proxy_in_vpn);
    assert!(!s.income_bypass);
    assert!(!s.income_hard_route);
    assert!(!s.income_access_from_lan);

    assert!(!s.routing_apps);
    assert_eq!(s.routing_unknown_tun_traffic, "Block");
    assert_eq!(s.routing_system_dns_tun, "Proxy");
    assert!(s.routing_dns_tun);
    assert!(!s.routing_dot_tun);
    assert!(s.routing_doh_tun);
    assert!(s.routing_bypass_lan);
    assert!(s.routing_bypass_lan_in_core);
    assert!(!s.routing_dpi);
    assert!(!s.routing_resolve_destination);
    assert!(!s.routing_ipv6_route);
    assert_eq!(s.routing_rules_source, "official");
    assert_eq!(s.routing_autoupdate_period_days, 7);

    // 2. Модификация параметров
    store.data.settings.income_access_from_lan = true;
    store.data.settings.income_local_ip = "0.0.0.0".to_string();
    store.data.settings.income_local_port = 10808;
    store.data.settings.routing_unknown_tun_traffic = "Proxy".to_string();
    store.data.settings.routing_rules_source = "Chocolate4U".to_string();
    store.save().expect("Save must succeed");

    // 3. Перезагрузка и проверка персистентности
    let reloaded = ConfigStore::load_or_default(Some(store_file));
    let rs = &reloaded.data.settings;
    assert!(rs.income_access_from_lan);
    assert_eq!(rs.income_local_ip, "0.0.0.0");
    assert_eq!(rs.income_local_port, 10808);
    assert_eq!(rs.routing_unknown_tun_traffic, "Proxy");
    assert_eq!(rs.routing_rules_source, "Chocolate4U");

    // 4. Обратная совместимость (пустой json со старыми полями)
    let legacy_json = serde_json::json!({
        "theme": "dark",
        "connection_mode": "Proxy",
        "mixed_port": 20808
    });
    let parsed: core_manager::AppSettings = serde_json::from_value(legacy_json).expect("Legacy settings must deserialize");
    assert_eq!(parsed.income_local_ip, "127.0.0.1");
    assert_eq!(parsed.income_local_port, 20808);
    assert_eq!(parsed.routing_rules_source, "official");
    assert_eq!(parsed.routing_unknown_tun_traffic, "Block");

    let _ = std::fs::remove_dir_all(temp_dir);
}


