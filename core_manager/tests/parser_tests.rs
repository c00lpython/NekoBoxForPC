//! Интеграционные и юнит-тесты на Rust для core_manager.

use core_manager::{
    detect_config_format, parse_config, to_singbox_json, AppConfig, ProfileConfig, ProtocolType,
};

#[test]
fn test_detect_all_formats() {
    let v2ray_link = "vless://12345678-1234-1234-1234-123456789abc@example.com:443?type=tcp&security=tls#VLESS";
    let clash_yaml = "proxies:\n  - name: test\n    type: ss\n    server: 1.1.1.1\n    port: 8388";
    let singbox_js = r#"{"outbounds": [{"type": "vless", "tag": "sb", "server": "1.1.1.1", "server_port": 443}]}"#;
    let xray_js = r#"{"outbounds": [{"protocol": "vless", "tag": "xray", "streamSettings": {"security": "reality"}}]}"#;
    let happ_link = "happ://happ.example.com:443#happ";
    let incy_link = "incy://incy.example.com:443#incy";
    let throne_link = "throne://throne.example.com:443#throne";

    assert_eq!(detect_config_format(v2ray_link), "v2ray");
    assert_eq!(detect_config_format(clash_yaml), "clash");
    assert_eq!(detect_config_format(singbox_js), "sing-box");
    assert_eq!(detect_config_format(xray_js), "xray");
    assert_eq!(detect_config_format(happ_link), "happ");
    assert_eq!(detect_config_format(incy_link), "incy");
    assert_eq!(detect_config_format(throne_link), "throne");
}

#[test]
fn test_parse_vless_uri() {
    let uri = "vless://12345678-1234-1234-1234-123456789abc@rust.example.com:443?type=tcp&security=tls#RustVLESS";
    let res = parse_config(uri);

    assert_eq!(res.format_name, "v2ray");
    assert_eq!(res.profiles.len(), 1);

    let p = &res.profiles[0];
    assert_eq!(p.name, "RustVLESS");
    assert_eq!(p.protocol, ProtocolType::Vless);
    assert_eq!(p.server, "rust.example.com");
    assert_eq!(p.server_port, 443);
}

#[test]
fn test_clash_yaml_parser() {
    let yaml = r#"
proxies:
  - name: "Clash Rust SS"
    type: ss
    server: ss.rust.com
    port: 8388
    cipher: aes-256-gcm
    password: rustpassword
"#;
    let res = parse_config(yaml);
    assert_eq!(res.format_name, "clash");
    assert_eq!(res.profiles.len(), 1);
    assert_eq!(res.profiles[0].name, "Clash Rust SS");
    assert_eq!(res.profiles[0].protocol, ProtocolType::Shadowsocks);
}

#[test]
fn test_singbox_json_parser() {
    let json_data = r#"{
        "outbounds": [
            {
                "type": "vless",
                "tag": "singbox-rust",
                "server": "sb.rust.com",
                "server_port": 443
            }
        ]
    }"#;
    let res = parse_config(json_data);
    assert_eq!(res.format_name, "sing-box");
    assert_eq!(res.profiles.len(), 1);
    assert_eq!(res.profiles[0].tag, "singbox-rust");
}

#[test]
fn test_happ_incy_throne_parsers() {
    let h = parse_config("happ://happ.rust.com:443#HappNode");
    assert_eq!(h.format_name, "happ");
    assert_eq!(h.profiles.len(), 1);

    let i = parse_config("incy://incy.rust.com:443#IncyNode");
    assert_eq!(i.format_name, "incy");
    assert_eq!(i.profiles.len(), 1);

    let t = parse_config("throne://throne.rust.com:443#ThroneNode");
    assert_eq!(t.format_name, "throne");
    assert_eq!(t.profiles.len(), 1);
}

#[test]
fn test_converter_to_singbox_json() {
    let p = ProfileConfig::new("Main Proxy", ProtocolType::Vless, "proxy.rust.com", 443);
    let app_config = AppConfig::default();

    let json_val = to_singbox_json(&[p], Some(&app_config), None);

    assert!(json_val.get("outbounds").is_some());
    assert!(json_val.get("inbounds").is_some());
    assert!(json_val.get("route").is_some());
    assert!(json_val.get("dns").is_some());

    let out0 = &json_val["outbounds"][0];
    assert_eq!(out0["type"], "vless");
    assert_eq!(out0["server"], "proxy.rust.com");
}
