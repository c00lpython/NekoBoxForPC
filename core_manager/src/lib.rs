//! NekoBox Core Manager на Rust.
//! Предоставляет универсальный парсер 7 форматов, утилиту задержек,
//! генератор Ultimate-конфига и супервайзер процесса ядра sing-box.

pub mod constructor;
pub mod converter;
pub mod latency;
pub mod models;
pub mod parser;
pub mod selector;
pub mod storage;
pub mod subscription;
pub mod supervisor;
pub mod template;
pub mod tui;

pub use constructor::{build_group_instant, build_group_interactive, build_profile_instant, build_profile_interactive};
pub use converter::to_singbox_json;
pub use latency::{AppSettings, CheckerPlugin, PluginRegistry, TcpPingPlugin, TlsPingPlugin, Http204Plugin, GeoIpPlugin};
pub use models::*;
pub use parser::{detect_config_format, parse_config, ParseResult};
pub use selector::{ParsedSelector, SelectorToken};
pub use storage::{
    BackupData, BackupMeta, BackupRestoreReport, ConfigStore, ConfigStoreData, StoredGeoFile,
    StoredGroup, StoredProfile, StoredRule,
};
pub use subscription::{FingerprintBuilder, RequestFingerprint, SubscriptionFetchResult, SubscriptionHandler, SubscriptionUserInfo};
pub use supervisor::CoreSupervisor;
pub use template::{inject_at_path, parse_path, get_default_master_template, UltimateConfigBuilder};


