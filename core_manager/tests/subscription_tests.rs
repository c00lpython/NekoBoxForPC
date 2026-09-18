//! Комплексные интеграционные тесты для подсистемы подписок.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use core_manager::{
    ConfigStore, FingerprintBuilder, ProtocolType, StoredGroup, StoredProfile, SubscriptionHandler,
    SubscriptionUserInfo,
};
use uuid::Uuid;

#[test]
fn test_subscription_userinfo_calculations() {
    let raw = "upload=2147483648; download=8589934592; total=107374182400; expire=1735689600";
    let info = SubscriptionUserInfo::parse(raw).expect("Should parse valid userinfo");

    assert_eq!(info.upload_bytes, 2147483648); // 2 GB
    assert_eq!(info.download_bytes, 8589934592); // 8 GB
    assert_eq!(info.total_bytes, 107374182400); // 100 GB
    assert_eq!(info.expire_timestamp, 1735689600);

    // used = 10 GB
    assert_eq!(info.used_bytes(), 10737418240);
    assert_eq!(SubscriptionUserInfo::format_bytes(info.used_bytes()), "10.00 GB");

    // percent = 10%
    assert_eq!(info.used_percent(), Some(10.0));

    // remaining = 90 GB
    assert_eq!(info.remaining_bytes(), Some(96636764160));
    assert_eq!(SubscriptionUserInfo::format_bytes(info.remaining_bytes().unwrap()), "90.00 GB");

    let sum = info.summary();
    assert!(sum.contains("10.00 GB / 100.00 GB (10.0%)"));
}

#[test]
fn test_subscription_userinfo_edge_cases() {
    // Безлимитный трафик (total=0)
    let raw = "upload=1048576; download=2097152; total=0";
    let info = SubscriptionUserInfo::parse(raw).expect("Should parse unlimited userinfo");
    assert_eq!(info.used_percent(), None);
    assert_eq!(info.remaining_bytes(), None);
    assert!(info.summary().contains("Безлимит"));

    // Пустой или мусорный заголовок
    assert!(SubscriptionUserInfo::parse("").is_none());
    assert!(SubscriptionUserInfo::parse("status=ok; test=1").is_none());
}

#[test]
fn test_fingerprint_builder_clients() {
    // 1. v2rayN (дефолт)
    let fp_v2ray = FingerprintBuilder::build("v2rayN", false);
    assert_eq!(fp_v2ray.user_agent, "v2rayN/6.23");
    assert!(!fp_v2ray.headers.contains_key("X-Hwid"));

    // 2. ClashMeta с включенным HWID
    let fp_clash = FingerprintBuilder::build("ClashMeta", true);
    assert_eq!(fp_clash.user_agent, "ClashMeta/v1.18.0");
    assert!(fp_clash.headers.contains_key("X-Hwid"));
    assert_eq!(fp_clash.headers.get("X-Device-Os").unwrap(), "Windows");

    // 3. Happ
    let fp_happ = FingerprintBuilder::build("happ", true);
    assert_eq!(fp_happ.user_agent, "Happ/3.26.3/Android/17839452147361875676");
    assert_eq!(fp_happ.headers.get("X-Device-Model").unwrap(), "PC-Desktop");

    // 4. v2raytun
    let fp_v2tun = FingerprintBuilder::build("v2raytun", false);
    assert_eq!(fp_v2tun.user_agent, "v2raytun/android");
    assert_eq!(fp_v2tun.headers.get("X-App-Version").unwrap(), "5.25.80");

    // 5. Incy
    let fp_incy = FingerprintBuilder::build("incy", false);
    assert_eq!(fp_incy.user_agent, "INCY/3.4.3/android Dalvik/2.1.0");
    assert_eq!(fp_incy.headers.get("X-Client").unwrap(), "INCY");

    // 6. HWID стабильность
    let hwid1 = FingerprintBuilder::generate_hwid();
    let hwid2 = FingerprintBuilder::generate_hwid();
    assert_eq!(hwid1, hwid2);
    assert_eq!(hwid1.len(), 32);
}

#[test]
fn test_decode_base64_and_plain_formats() {
    // Base64 со списком URI
    let links = "vless://00000000-0000-0000-0000-000000000001@1.1.1.1:443?security=tls#VlessNode\nss://YWVzLTEyOC1nY206cGFzc3dvcmRAMS4yLjMuNDo4Mzg4#ShadowsocksNode";
    let encoded = STANDARD.encode(links);

    let decoded = SubscriptionHandler::decode_payload_if_base64(&encoded);
    assert_eq!(decoded, links);

    // Sing-box JSON не должен декодироваться как base64
    let singbox_json = r#"{"outbounds":[{"type":"vless","tag":"sb-node","server":"2.2.2.2","server_port":443}]}"#;
    assert_eq!(SubscriptionHandler::decode_payload_if_base64(singbox_json), singbox_json);

    // Clash YAML не должен декодироваться
    let clash_yaml = "proxies:\n  - name: clash-node\n    type: vmess\n    server: 3.3.3.3\n    port: 443\n    uuid: abc\n    alterId: 0\n    cipher: auto";
    assert_eq!(SubscriptionHandler::decode_payload_if_base64(clash_yaml), clash_yaml);
}

#[test]
fn test_storage_subscription_metadata_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("nb4a_sub_store_{}", Uuid::new_v4()));
    let store_file = temp_dir.join("store.json");

    let mut store = ConfigStore::load_or_default(Some(store_file.clone()));

    let mut sub_group = StoredGroup::new_subscription("MySub", "https://api.example.com/sub/token");
    sub_group.client_imitation = "ClashMeta".to_string();
    sub_group.hwid = true;
    sub_group.auto_update_minutes = 120;
    sub_group.subscription_userinfo = Some(SubscriptionUserInfo {
        upload_bytes: 1000,
        download_bytes: 5000,
        total_bytes: 50000,
        expire_timestamp: 1800000000,
    });
    sub_group.last_updated_at = Some(1726000000);

    // Добавляем тестовый профиль в группу
    sub_group.profiles.push(StoredProfile {
        id: "p-sub-1".to_string(),
        name: "node-sub-1".to_string(),
        protocol: ProtocolType::Vless,
        server: "10.20.30.40".to_string(),
        server_port: 443,
        settings: serde_json::json!({ "type": "vless" }),
        tag: "node-sub-1".to_string(),
        last_ping_ms: Some(15),
    });

    store.add_group(sub_group).expect("Failed to add sub group");

    // Перезагрузка хранилища с диска
    let reloaded = ConfigStore::load_or_default(Some(store_file));
    let loaded_group = reloaded.get_group("MySub").expect("Group should exist");

    assert_eq!(loaded_group.group_type, "sub");
    assert_eq!(loaded_group.subscription_url.as_deref(), Some("https://api.example.com/sub/token"));
    assert_eq!(loaded_group.client_imitation, "ClashMeta");
    assert!(loaded_group.hwid);
    assert_eq!(loaded_group.auto_update_minutes, 120);
    assert_eq!(loaded_group.last_updated_at, Some(1726000000));

    let userinfo = loaded_group.subscription_userinfo.as_ref().expect("Userinfo should persist");
    assert_eq!(userinfo.upload_bytes, 1000);
    assert_eq!(userinfo.download_bytes, 5000);
    assert_eq!(userinfo.total_bytes, 50000);
    assert_eq!(userinfo.expire_timestamp, 1800000000);

    assert_eq!(loaded_group.profiles.len(), 1);
    assert_eq!(loaded_group.profiles[0].name, "node-sub-1");

    // Очистка
    let _ = std::fs::remove_dir_all(temp_dir);
}
