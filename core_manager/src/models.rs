//! Доменные модели конфигурации NekoBox на Rust.
//! Описывают профили, группы, настройки TUN, DNS, инбаунды и маршрутизацию.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    Shadowsocks,
    Vmess,
    Vless,
    Trojan,
    Hysteria2,
    Wireguard,
    Amneziawg,
    Tuic,
    Ssh,
    Byedpi,
    Direct,
    Block,
    Dns,
    Balancer,
    Chain,
    MasterDnsVPN,
}

impl ProtocolType {
    pub fn as_str(&self) -> &'static str {
        match self {
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
            ProtocolType::Direct => "direct",
            ProtocolType::Block => "block",
            ProtocolType::Dns => "dns",
            ProtocolType::Balancer => "balancer",
            ProtocolType::Chain => "chain",
            ProtocolType::MasterDnsVPN => "masterdnsvpn",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "shadowsocks" | "ss" => ProtocolType::Shadowsocks,
            "vmess" => ProtocolType::Vmess,
            "vless" => ProtocolType::Vless,
            "trojan" => ProtocolType::Trojan,
            "hysteria2" | "hy2" => ProtocolType::Hysteria2,
            "wireguard" | "wg" => ProtocolType::Wireguard,
            "amneziawg" | "awg" => ProtocolType::Amneziawg,
            "tuic" => ProtocolType::Tuic,
            "ssh" => ProtocolType::Ssh,
            "byedpi" | "dpi" => ProtocolType::Byedpi,
            "masterdnsvpn" | "masterdns" | "master-dns" => ProtocolType::MasterDnsVPN,
            "direct" => ProtocolType::Direct,
            "block" => ProtocolType::Block,
            "dns" => ProtocolType::Dns,
            "balancer" | "urltest" | "selector" => ProtocolType::Balancer,
            "chain" | "proxychain" => ProtocolType::Chain,
            _ => ProtocolType::Vless,
        }
    }
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ProtocolType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ProtocolType::from_str(s))
    }
}

impl Default for ProtocolType {
    fn default() -> Self {
        ProtocolType::Vless
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TUNStack {
    System,
    Gvisor,
    Mixed,
}

impl Default for TUNStack {
    fn default() -> Self {
        TUNStack::Mixed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoutingMode {
    Global,
    Rule,
    Direct,
}

impl Default for RoutingMode {
    fn default() -> Self {
        RoutingMode::Rule
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub protocol: ProtocolType,
    pub server: String,
    pub server_port: u16,
    pub settings: serde_json::Value,
    pub tag: String,
    pub group_id: Option<String>,
    pub latency_ms: Option<f64>,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub extra_byedpi_preset: Option<String>,
}

impl ProfileConfig {
    pub fn new(name: impl Into<String>, protocol: ProtocolType, server: impl Into<String>, server_port: u16) -> Self {
        let id = Uuid::new_v4().to_string();
        let tag = format!("proxy-{}", &id[..8]);
        Self {
            id,
            name: name.into(),
            protocol,
            server: server.into(),
            server_port,
            settings: serde_json::Value::Object(serde_json::Map::new()),
            tag,
            group_id: None,
            latency_ms: None,
            upload_bytes: 0,
            download_bytes: 0,
            extra_byedpi_preset: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub id: String,
    pub name: String,
    pub subscription_url: Option<String>,
    pub auto_update_minutes: u32,
    pub last_updated: Option<String>,
    pub profiles: Vec<ProfileConfig>,
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Default Group".to_string(),
            subscription_url: None,
            auto_update_minutes: 0,
            last_updated: None,
            profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteRule {
    pub outbound: String,
    pub domains: Vec<String>,
    pub domain_suffixes: Vec<String>,
    pub domain_keywords: Vec<String>,
    pub ip_cidrs: Vec<String>,
    pub geoip: Vec<String>,
    pub geosite: Vec<String>,
    pub port_ranges: Vec<String>,
    pub process_names: Vec<String>,
    pub rule_set: Vec<String>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TUNConfig {
    pub enabled: bool,
    pub interface_name: String,
    pub inet4_address: String,
    pub inet6_address: String,
    pub auto_route: bool,
    pub strict_route: bool,
    pub stack: TUNStack,
    pub mtu: u32,
    pub endpoint_independent_nat: bool,
}

impl Default for TUNConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interface_name: "nekobox-tun".to_string(),
            inet4_address: "172.19.0.1/30".to_string(),
            inet6_address: "fdfe:dcba:9876::1/126".to_string(),
            auto_route: true,
            strict_route: false,
            stack: TUNStack::Mixed,
            mtu: 9000,
            endpoint_independent_nat: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundSettings {
    pub mixed_port: u16,
    pub socks_port: u16,
    pub http_port: u16,
    pub clash_api_port: u16,
    pub clash_secret: String,
    pub v2ray_api_port: u16,
    pub allow_lan: bool,
}

impl Default for InboundSettings {
    fn default() -> Self {
        Self {
            mixed_port: 2080,
            socks_port: 2081,
            http_port: 2082,
            clash_api_port: 9090,
            clash_secret: "".to_string(),
            v2ray_api_port: 10085,
            allow_lan: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub routing_mode: RoutingMode,
    pub inbound: InboundSettings,
    pub tun: TUNConfig,
    pub rules: Vec<RouteRule>,
    pub groups: Vec<GroupConfig>,
    pub active_profile_id: Option<String>,
    pub adblock_enabled: bool,
    pub masterdnsvpn_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            routing_mode: RoutingMode::Rule,
            inbound: InboundSettings::default(),
            tun: TUNConfig::default(),
            rules: Vec::new(),
            groups: Vec::new(),
            active_profile_id: None,
            adblock_enabled: true,
            masterdnsvpn_enabled: true,
        }
    }
}
