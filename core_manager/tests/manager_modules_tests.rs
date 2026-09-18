//! Интеграционные тесты для 4 новых модулей:
//! утилиты проверок задержки, генератора Ultimate-конфига и супервайзера.

use core_manager::latency::{
    AppSettings, CheckResult, CheckerPlugin, PluginRegistry,
};
use core_manager::template::{
    inject_at_path, parse_path, PathSegment, UltimateConfigBuilder,
};
use core_manager::{CoreSupervisor, ProfileConfig, ProtocolType};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_path_parser_bracket_and_dot_syntax() {
    let bracket = parse_path("[outbounds][0][server]");
    assert_eq!(
        bracket,
        vec![
            PathSegment::Key("outbounds".into()),
            PathSegment::Index(0),
            PathSegment::Key("server".into())
        ]
    );

    let dot = parse_path("route.rules.1.outbound");
    assert_eq!(
        dot,
        vec![
            PathSegment::Key("route".into()),
            PathSegment::Key("rules".into()),
            PathSegment::Index(1),
            PathSegment::Key("outbound".into())
        ]
    );
}

#[test]
fn test_inject_at_path_nested() {
    let mut root = json!({
        "outbounds": [
            {"type": "direct", "tag": "direct"}
        ]
    });

    // Инъекция в [outbounds][0]
    let new_outbound = json!({
        "type": "vless",
        "tag": "proxy",
        "server": "example.com",
        "server_port": 443
    });

    inject_at_path(&mut root, "[outbounds][0]", new_outbound.clone()).unwrap();
    assert_eq!(root["outbounds"][0]["type"], "vless");
    assert_eq!(root["outbounds"][0]["server"], "example.com");

    // Инъекция глубокого нового поля
    inject_at_path(&mut root, "[outbounds][0][tls][enabled]", json!(true)).unwrap();
    assert_eq!(root["outbounds"][0]["tls"]["enabled"], true);
}

#[test]
fn test_ultimate_config_builder() {
    let profile = ProfileConfig::new("Test Proxy", ProtocolType::Vless, "vless.test.com", 8443);
    let mut builder = UltimateConfigBuilder::new();

    builder.inject_profile(&profile).unwrap();
    builder.set_tun_enabled(false).unwrap();
    builder.set_clash_api("127.0.0.1:9095", "custom_ui").unwrap();

    let config = builder.build();

    // Проверяем [outbounds][0]
    let out0 = &config["outbounds"][0];
    assert_eq!(out0["type"], "vless");
    assert_eq!(out0["server"], "vless.test.com");

    // Проверяем TUN выключен
    let inbounds = config["inbounds"].as_array().unwrap();
    assert!(inbounds.iter().all(|i| i["type"] != "tun"));

    // Проверяем Clash API
    assert_eq!(
        config["experimental"]["clash_api"]["external_controller"],
        "127.0.0.1:9095"
    );
    assert_eq!(
        config["experimental"]["clash_api"]["external_ui"],
        "custom_ui"
    );
}

#[tokio::test]
async fn test_plugin_registry_custom_plugin() {
    struct MockPingPlugin;
    impl CheckerPlugin for MockPingPlugin {
        fn name(&self) -> &'static str {
            "mock_plugin"
        }
        fn check<'a>(
            &'a self,
            _profile: &'a ProfileConfig,
            _settings: &'a AppSettings,
        ) -> core_manager::latency::BoxFuture<'a, Result<CheckResult, String>> {
            Box::pin(async move { Ok(CheckResult::LatencyMs(42)) })
        }
    }

    let mut registry = PluginRegistry::new();
    registry.register(Arc::new(MockPingPlugin));

    let profile = ProfileConfig::new("Mock", ProtocolType::Shadowsocks, "127.0.0.1", 1080);
    let settings = AppSettings::default();

    let report = registry.run_all(&profile, &settings).await;
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].0, "mock_plugin");
    match &report.results[0].1 {
        Ok(CheckResult::LatencyMs(ms)) => assert_eq!(*ms, 42),
        _ => panic!("Expected LatencyMs(42)"),
    }
}

#[test]
fn test_core_supervisor_discovery() {
    // Проверка автоматического обнаружения исполняемого файла ядра
    let path = CoreSupervisor::find_default_core_path();
    assert!(path.is_some(), "Бинарник singbox должен быть найден в проекте");
}
