//! Модуль постоянного хранения групп, профилей и активного состояния.

use crate::models::{ProfileConfig, ProtocolType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProfile {
    pub id: String,
    pub name: String,
    pub protocol: ProtocolType,
    pub server: String,
    pub server_port: u16,
    pub settings: serde_json::Value,
    pub tag: String,
    pub last_ping_ms: Option<u64>,
}

impl From<ProfileConfig> for StoredProfile {
    fn from(p: ProfileConfig) -> Self {
        Self {
            id: p.id,
            name: p.name,
            protocol: p.protocol,
            server: p.server,
            server_port: p.server_port,
            settings: p.settings,
            tag: p.tag,
            last_ping_ms: p.latency_ms.map(|ms| ms as u64),
        }
    }
}

impl From<StoredProfile> for ProfileConfig {
    fn from(sp: StoredProfile) -> Self {
        Self {
            id: sp.id,
            name: sp.name,
            protocol: sp.protocol,
            server: sp.server,
            server_port: sp.server_port,
            settings: sp.settings,
            tag: sp.tag,
            group_id: None,
            latency_ms: sp.last_ping_ms.map(|ms| ms as f64),
            upload_bytes: 0,
            download_bytes: 0,
            extra_byedpi_preset: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGroup {
    pub name: String,
    pub group_type: String, // "manual" или "sub"
    pub subscription_url: Option<String>,
    pub auto_update_minutes: u32,
    pub hwid: bool,
    pub client_imitation: String,
    pub front_proxy: Option<String>,
    pub outbound_proxy: Option<String>,
    pub profiles: Vec<StoredProfile>,
    #[serde(default)]
    pub subscription_userinfo: Option<crate::subscription::SubscriptionUserInfo>,
    #[serde(default)]
    pub last_updated_at: Option<u64>,
    #[serde(default)]
    pub error_message: Option<String>,
}

impl StoredGroup {
    pub fn new_manual(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            group_type: "manual".to_string(),
            subscription_url: None,
            auto_update_minutes: 0,
            hwid: false,
            client_imitation: "v2rayN".to_string(),
            front_proxy: None,
            outbound_proxy: None,
            profiles: Vec::new(),
            subscription_userinfo: None,
            last_updated_at: None,
            error_message: None,
        }
    }

    pub fn new_subscription(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            group_type: "sub".to_string(),
            subscription_url: Some(url.into()),
            auto_update_minutes: 60,
            hwid: false,
            client_imitation: "v2rayN".to_string(),
            front_proxy: None,
            outbound_proxy: None,
            profiles: Vec::new(),
            subscription_userinfo: None,
            last_updated_at: None,
            error_message: None,
        }
    }

}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRule {
    pub name: String,
    pub action: String, // "direct", "proxy", "block", "dns"
    #[serde(default)]
    pub site: Vec<String>,
    #[serde(default)]
    pub ip: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    // Interface
    #[serde(default = "default_language")]
    pub language: String,
    pub theme: String,
    pub show_icons: bool,
    pub show_country: bool,
    pub display_address: bool,
    // Connection
    pub connection_mode: String, // "Proxy", "VPN"
    pub stack: String, // "mixed", "gvisor", "system"
    // Core
    pub mux_protocol: String, // "h2mux", "yamux", "s2mux"
    pub memory_limit_mb: u32,
    pub cert_validation: bool,
    // Income (Входящие)
    #[serde(default = "default_true")]
    pub income_local_proxy_in_vpn: bool,
    #[serde(default = "default_false")]
    pub income_turn_off_udp: bool,
    #[serde(default = "default_income_local_ip")]
    pub income_local_ip: String,
    #[serde(default = "default_income_local_port")]
    pub income_local_port: u16,
    #[serde(default = "default_empty_string")]
    pub income_username: String,
    #[serde(default = "default_empty_string")]
    pub income_password: String,
    #[serde(default = "default_true")]
    pub income_http_proxy_in_vpn: bool,
    #[serde(default = "default_false")]
    pub income_bypass: bool,
    #[serde(default = "default_false")]
    pub income_hard_route: bool,
    #[serde(default = "default_false")]
    pub income_access_from_lan: bool,
    // Routing (Маршрутизация)
    #[serde(default = "default_false")]
    pub routing_apps: bool,
    #[serde(default = "default_routing_unknown_tun")]
    pub routing_unknown_tun_traffic: String,
    #[serde(default = "default_routing_system_dns_tun")]
    pub routing_system_dns_tun: String,
    #[serde(default = "default_true")]
    pub routing_dns_tun: bool,
    #[serde(default = "default_false")]
    pub routing_dot_tun: bool,
    #[serde(default = "default_true")]
    pub routing_doh_tun: bool,
    #[serde(default = "default_true")]
    pub routing_bypass_lan: bool,
    #[serde(default = "default_true")]
    pub routing_bypass_lan_in_core: bool,
    #[serde(default = "default_false")]
    pub routing_dpi: bool,
    #[serde(default = "default_false")]
    pub routing_resolve_destination: bool,
    #[serde(default = "default_false")]
    pub routing_ipv6_route: bool,
    #[serde(default = "default_routing_rules_source")]
    pub routing_rules_source: String,
    #[serde(default = "default_routing_autoupdate_period")]
    pub routing_autoupdate_period_days: u32,
    // DNS
    pub dns_routing_enabled: bool,
    pub fake_dns: bool,
    pub direct_dns: String,
    pub proxy_dns: String,
    pub user_dns: Vec<String>,
    pub enable_ipv6: bool,
    pub geo_resource: String,
    // Tests
    pub test_url: String,
    pub geo_url: String,
    pub speedtest_url: String,
    pub test_timeout_ms: u64,
    pub test_type: String, // "GET", "HEAD", "TCP", "RTT"
    // Logging
    pub log_level: String,
    pub log_save_path: String,
    // Other
    pub utls_fingerprint: String,
    pub utls_min_version: String,
    pub autoupdate_subs_default: bool,
    pub autoupdate_resources: bool,
    pub rule_resources_url: String,
    // Latency & Ports
    #[serde(default = "default_tcp_timeout")]
    pub tcp_ping_timeout_ms: u64,
    #[serde(default = "default_tls_timeout")]
    pub tls_ping_timeout_ms: u64,
    #[serde(default = "default_http_timeout")]
    pub http_204_timeout_ms: u64,
    #[serde(default = "default_http_url")]
    pub http_204_url: String,
    #[serde(default = "default_geo_url")]
    pub geoip_api_url: String,
    #[serde(default = "default_concurrency")]
    pub max_concurrency: usize,
    #[serde(default = "default_clash_port")]
    pub clash_api_port: u16,
    #[serde(default = "default_mixed_port")]
    pub mixed_port: u16,
}

fn default_language() -> String { "ru".to_string() }
fn default_tcp_timeout() -> u64 { 2000 }
fn default_tls_timeout() -> u64 { 3000 }
fn default_http_timeout() -> u64 { 3500 }
fn default_http_url() -> String { "https://www.gstatic.com/generate_204".to_string() }
fn default_geo_url() -> String { "https://ipinfo.io/json".to_string() }
fn default_concurrency() -> usize { 30 }
fn default_clash_port() -> u16 { 9090 }
fn default_mixed_port() -> u16 { 20808 }
fn default_false() -> bool { false }
fn default_income_local_ip() -> String { "127.0.0.1".to_string() }
fn default_income_local_port() -> u16 { 20808 }
fn default_empty_string() -> String { String::new() }
fn default_routing_unknown_tun() -> String { "Block".to_string() }
fn default_routing_system_dns_tun() -> String { "Proxy".to_string() }
fn default_routing_rules_source() -> String { "official".to_string() }
fn default_routing_autoupdate_period() -> u32 { 7 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "ru".to_string(),
            theme: "dark".to_string(),
            show_icons: true,
            show_country: true,
            display_address: true,
            connection_mode: "Proxy".to_string(),
            stack: "mixed".to_string(),
            mux_protocol: "h2mux".to_string(),
            memory_limit_mb: 512,
            cert_validation: true,
            income_local_proxy_in_vpn: true,
            income_turn_off_udp: false,
            income_local_ip: "127.0.0.1".to_string(),
            income_local_port: 20808,
            income_username: "".to_string(),
            income_password: "".to_string(),
            income_http_proxy_in_vpn: true,
            income_bypass: false,
            income_hard_route: false,
            income_access_from_lan: false,
            routing_apps: false,
            routing_unknown_tun_traffic: "Block".to_string(),
            routing_system_dns_tun: "Proxy".to_string(),
            routing_dns_tun: true,
            routing_dot_tun: false,
            routing_doh_tun: true,
            routing_bypass_lan: true,
            routing_bypass_lan_in_core: true,
            routing_dpi: false,
            routing_resolve_destination: false,
            routing_ipv6_route: false,
            routing_rules_source: "official".to_string(),
            routing_autoupdate_period_days: 7,
            dns_routing_enabled: true,
            fake_dns: false,
            direct_dns: "77.88.8.8".to_string(),
            proxy_dns: "https://1.1.1.1/dns-query".to_string(),
            user_dns: vec!["8.8.8.8".to_string()],
            enable_ipv6: false,
            geo_resource: "geoip.db".to_string(),
            test_url: "https://www.gstatic.com/generate_204".to_string(),
            geo_url: "https://ipinfo.io/json".to_string(),
            speedtest_url: "https://speed.cloudflare.com/__down?bytes=10000000".to_string(),
            test_timeout_ms: 3000,
            test_type: "HTTP".to_string(),
            log_level: "trace".to_string(),
            log_save_path: "logs/latest.log".to_string(),
            utls_fingerprint: "chrome".to_string(),
            utls_min_version: "1.2".to_string(),
            autoupdate_subs_default: true,
            autoupdate_resources: true,
            rule_resources_url: "".to_string(),
            tcp_ping_timeout_ms: 2000,
            tls_ping_timeout_ms: 3000,
            http_204_timeout_ms: 3500,
            http_204_url: "https://www.gstatic.com/generate_204".to_string(),
            geoip_api_url: "https://ipinfo.io/json".to_string(),
            max_concurrency: 30,
            clash_api_port: 9090,
            mixed_port: 20808,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGeoFile {
    pub name: String,
    pub file_type: String, // "geoip", "geosite", "rule-set"
    pub source_url: Option<String>,
    pub local_path: String,
    pub size_bytes: u64,
    pub updated_at: u64,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    pub name: String,
    pub created_at: u64,
    pub app_version: String,
    pub includes_configs: bool,
    pub includes_routes: bool,
    pub includes_settings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub meta: BackupMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_picked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<StoredGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<StoredRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<AppSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_files: Option<Vec<StoredGeoFile>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMode {
    Merge,
    Hard,
}

impl RecoveryMode {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "hard" | "force" | "overwrite" => RecoveryMode::Hard,
            _ => RecoveryMode::Merge,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupPreviewItem {
    pub name: String,
    pub profiles_count: usize,
    pub action_desc: String,
}

#[derive(Debug, Clone)]
pub struct RulePreviewItem {
    pub name: String,
    pub action: String,
    pub action_desc: String,
}

#[derive(Debug, Clone)]
pub struct RecoveryPreview {
    pub backup_name: String,
    pub backup_version: String,
    pub mode: RecoveryMode,
    pub apply_configs: bool,
    pub apply_routes: bool,
    pub apply_settings: bool,
    pub groups: Vec<GroupPreviewItem>,
    pub rules: Vec<RulePreviewItem>,
    pub settings_diffs: Vec<String>,
    pub geo_files_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct BackupRestoreReport {
    pub configs_restored: bool,
    pub groups_count: usize,
    pub profiles_count: usize,
    pub routes_restored: bool,
    pub rules_count: usize,
    pub settings_restored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigStoreData {
    pub active_picked: Option<String>,
    pub last_run: Option<String>,
    pub groups: Vec<StoredGroup>,
    #[serde(default)]
    pub rules: Vec<StoredRule>,
    #[serde(default)]
    pub settings: AppSettings,
    #[serde(default)]
    pub geo_files: Vec<StoredGeoFile>,
}

/// Возвращает базовый каталог данных приложения `data/` в папке исполняемого файла.
/// Если путь к exe недоступен, используется `./data`.
pub fn get_data_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let p = exe_dir.join("data");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }
    let p = PathBuf::from("data");
    let _ = fs::create_dir_all(&p);
    p
}

pub struct ConfigStore {
    path: PathBuf,
    pub data: ConfigStoreData,
}

impl ConfigStore {
    pub fn default_path() -> PathBuf {
        let data_dir = get_data_dir();
        // Автоматическая миграция со старого .runtime/store.json, если новый еще не существует
        let legacy_store = PathBuf::from(".runtime").join("store.json");
        let target_store = data_dir.join("store.json");
        if !target_store.exists() && legacy_store.exists() {
            if let Ok(content) = fs::read_to_string(&legacy_store) {
                let _ = fs::write(&target_store, content);
            }
        }
        target_store
    }

    pub fn load_or_default(custom_path: Option<PathBuf>) -> Self {
        let path = custom_path.unwrap_or_else(Self::default_path);

        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<ConfigStoreData>(&content) {
                    return Self { path, data };
                }
            }
        }

        // Инициализация дефолтной пустой группы "Default"
        let mut data = ConfigStoreData::default();
        data.groups.push(StoredGroup::new_manual("Default"));

        Self { path, data }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json_str = serde_json::to_string_pretty(&self.data)
            .map_err(|e| format!("Ошибка сериализации хранилища: {}", e))?;

        fs::write(&self.path, json_str)
            .map_err(|e| format!("Ошибка сохранения хранилища в {}: {}", self.path.display(), e))
    }

    // --- Управление группами ---

    pub fn get_group(&self, name: &str) -> Option<&StoredGroup> {
        self.data.groups.iter().find(|g| g.name.eq_ignore_ascii_case(name))
    }

    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut StoredGroup> {
        self.data.groups.iter_mut().find(|g| g.name.eq_ignore_ascii_case(name))
    }

    pub fn add_group(&mut self, group: StoredGroup) -> Result<(), String> {
        if self.get_group(&group.name).is_some() {
            return Err(format!("Группа '{}' уже существует", group.name));
        }
        self.data.groups.push(group);
        self.save()
    }

    pub fn remove_group(&mut self, name: &str) -> Result<(), String> {
        let initial_len = self.data.groups.len();
        self.data.groups.retain(|g| !g.name.eq_ignore_ascii_case(name));
        if self.data.groups.len() == initial_len {
            return Err(format!("Группа '{}' не найдена", name));
        }
        self.save()
    }

    // --- Управление профилями ---

    pub fn add_profile(&mut self, group_name: &str, profile: StoredProfile) -> Result<(), String> {
        let group = self
            .get_group_mut(group_name)
            .ok_or_else(|| format!("Группа '{}' не найдена", group_name))?;

        // Замена если профиль с таким именем уже есть, иначе добавление
        if let Some(existing) = group.profiles.iter_mut().find(|p| p.name == profile.name) {
            *existing = profile;
        } else {
            group.profiles.push(profile);
        }
        self.save()
    }

    pub fn remove_profile(&mut self, target: &str) -> Result<(), String> {
        let (group_name, profile_name) = self.split_target(target)?;
        let group = self
            .get_group_mut(&group_name)
            .ok_or_else(|| format!("Группа '{}' не найдена", group_name))?;

        let initial_len = group.profiles.len();
        group.profiles.retain(|p| p.name != profile_name);

        if group.profiles.len() == initial_len {
            return Err(format!("Профиль '{}' не найден в группе '{}'", profile_name, group_name));
        }

        // Если удалили выбранный активный профиль — сбрасываем pick
        if self.data.active_picked.as_deref() == Some(target) {
            self.data.active_picked = None;
        }

        self.save()
    }

    pub fn find_profile(&self, target: &str) -> Option<(&StoredGroup, &StoredProfile)> {
        if target.contains('.') {
            let mut parts = target.splitn(2, '.');
            let grp_name = parts.next()?;
            let prof_name = parts.next()?;
            let grp = self.get_group(grp_name)?;
            let prof = grp.profiles.iter().find(|p| p.name.eq_ignore_ascii_case(prof_name))?;
            Some((grp, prof))
        } else {
            // Поиск по всем группам по имени профиля
            for grp in &self.data.groups {
                if let Some(prof) = grp.profiles.iter().find(|p| p.name.eq_ignore_ascii_case(target)) {
                    return Some((grp, prof));
                }
            }
            None
        }
    }

    pub fn update_ping(&mut self, target: &str, ping_ms: u64) -> Result<(), String> {
        let (group_name, profile_name) = self.split_target(target)?;
        let group = self
            .get_group_mut(&group_name)
            .ok_or_else(|| format!("Группа '{}' не найдена", group_name))?;

        let prof = group
            .profiles
            .iter_mut()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| format!("Профиль '{}' не найден", profile_name))?;

        prof.last_ping_ms = Some(ping_ms);
        self.save()
    }

    pub fn update_pings_batch(&mut self, updates: &[(String, String, Option<u64>)]) -> Result<usize, String> {
        let mut count = 0;
        for (grp_name, prof_name, ping_ms) in updates {
            if let Some(group) = self.get_group_mut(grp_name) {
                if let Some(prof) = group.profiles.iter_mut().find(|p| p.name == *prof_name) {
                    prof.last_ping_ms = *ping_ms;
                    count += 1;
                }
            }
        }
        if count > 0 {
            self.save()?;
        }
        Ok(count)
    }

    // --- State: Pick, Unpick, Resolve Run ---

    pub fn pick(&mut self, target: &str) -> Result<(), String> {
        let (grp, prof) = self
            .find_profile(target)
            .ok_or_else(|| format!("Профиль '{}' не найден", target))?;
        let full_target = format!("{}.{}", grp.name, prof.name);
        self.data.active_picked = Some(full_target);
        self.save()
    }

    pub fn unpick(&mut self) -> Result<(), String> {
        self.data.active_picked = None;
        self.save()
    }

    pub fn resolve_run_target(
        &mut self,
        explicit_target: Option<&str>,
    ) -> Option<(StoredGroup, StoredProfile)> {
        if let Some(tgt) = explicit_target {
            if let Some((g, p)) = self.find_profile(tgt) {
                let res = (g.clone(), p.clone());
                self.data.last_run = Some(format!("{}.{}", g.name, p.name));
                let _ = self.save();
                return Some(res);
            }
        }

        // 1. Попытка взять picked
        if let Some(picked) = &self.data.active_picked {
            if let Some((g, p)) = self.find_profile(picked) {
                let res = (g.clone(), p.clone());
                self.data.last_run = Some(format!("{}.{}", g.name, p.name));
                let _ = self.save();
                return Some(res);
            }
        }

        // 2. Попытка взять last_run
        if let Some(last) = &self.data.last_run {
            if let Some((g, p)) = self.find_profile(last) {
                return Some((g.clone(), p.clone()));
            }
        }

        // 3. Fallback: первый доступный профиль в первой непустой группе
        for g in &self.data.groups {
            if let Some(p) = g.profiles.first() {
                let res = (g.clone(), p.clone());
                self.data.last_run = Some(format!("{}.{}", g.name, p.name));
                let _ = self.save();
                return Some(res);
            }
        }

        None
    }

    fn split_target(&self, target: &str) -> Result<(String, String), String> {
        if target.contains('.') {
            let mut parts = target.splitn(2, '.');
            let grp = parts.next().unwrap().to_string();
            let prof = parts.next().unwrap().to_string();
            Ok((grp, prof))
        } else {
            // Если указано только имя профиля, ищем в какой группе он находится
            if let Some((grp, prof)) = self.find_profile(target) {
                Ok((grp.name.clone(), prof.name.clone()))
            } else {
                Ok(("Default".to_string(), target.to_string()))
            }
        }
    }

    // --- Управление плоским списком профилей по индексам (1-based) ---

    pub fn get_flat_profiles(&self) -> Vec<(&StoredGroup, &StoredProfile)> {
        let mut flat = Vec::new();
        for grp in &self.data.groups {
            for prof in &grp.profiles {
                flat.push((grp, prof));
            }
        }
        flat
    }

    pub fn find_profile_by_index_or_target(&self, query: &str) -> Option<(&StoredGroup, &StoredProfile)> {
        if let Ok(idx) = query.parse::<usize>() {
            if idx >= 1 {
                let flat = self.get_flat_profiles();
                if idx <= flat.len() {
                    return Some(flat[idx - 1]);
                }
            }
        }
        if query.contains('.') {
            let mut parts = query.splitn(2, '.');
            let grp_name = parts.next()?;
            let right = parts.next()?;
            if let Ok(idx) = right.parse::<usize>() {
                if let Some(grp) = self.get_group(grp_name) {
                    if idx >= 1 && idx <= grp.profiles.len() {
                        return Some((grp, &grp.profiles[idx - 1]));
                    }
                }
            }
        }
        self.find_profile(query)
    }

    // --- Управление правилами маршрутизации ---

    pub fn add_rule(&mut self, name: &str, action: &str, site: Vec<String>, ip: Vec<String>) -> Result<(), String> {
        if self.data.rules.iter().any(|r| r.name.eq_ignore_ascii_case(name)) {
            return Err(format!("Правило с именем '{}' уже существует", name));
        }
        self.data.rules.push(StoredRule {
            name: name.to_string(),
            action: action.to_string(),
            site,
            ip,
            enabled: true,
        });
        self.save()
    }

    pub fn update_rule(&mut self, name: &str, updated: StoredRule) -> Result<(), String> {
        let idx = self.data.rules.iter().position(|r| r.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("Правило '{}' не найдено", name))?;
        self.data.rules[idx] = updated;
        self.save()
    }

    pub fn del_rule(&mut self, name: &str) -> Result<(), String> {
        let init_len = self.data.rules.len();
        self.data.rules.retain(|r| !r.name.eq_ignore_ascii_case(name));
        if self.data.rules.len() == init_len {
            return Err(format!("Правило '{}' не найдено", name));
        }
        self.save()
    }

    pub fn move_rule(&mut self, name: &str, new_pos: usize) -> Result<(), String> {
        let idx = self.data.rules.iter().position(|r| r.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("Правило '{}' не найдено", name))?;
        
        let rule = self.data.rules.remove(idx);
        let target_idx = if new_pos == 0 { 0 } else { (new_pos - 1).min(self.data.rules.len()) };
        self.data.rules.insert(target_idx, rule);
        self.save()
    }

    pub fn switch_rule(&mut self, name: &str, wanted_state: Option<bool>) -> Result<bool, String> {
        let rule = self.data.rules.iter_mut().find(|r| r.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("Правило '{}' не найдено", name))?;
        
        let new_state = wanted_state.unwrap_or(!rule.enabled);
        rule.enabled = new_state;
        let res = rule.enabled;
        self.save()?;
        Ok(res)
    }

    // --- Управление гео-файлами в LocalStorage ---

    pub fn get_geo_dir(&self) -> PathBuf {
        let base = self.path.parent().map(|p| p.to_path_buf()).unwrap_or_else(get_data_dir);
        let dir = base.join("geo");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn list_geo_files(&self) -> &[StoredGeoFile] {
        &self.data.geo_files
    }

    pub fn save_geo_bytes(
        &mut self,
        name: &str,
        file_type: &str,
        source_url: Option<String>,
        data: &[u8],
    ) -> Result<StoredGeoFile, String> {
        let geo_dir = self.get_geo_dir();
        let target_file = geo_dir.join(name);

        fs::write(&target_file, data)
            .map_err(|e| format!("Ошибка записи гео-файла '{}': {}", target_file.display(), e))?;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let sha256 = hex::encode(hasher.finalize());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let rel_path = format!("geo/{}", name);
        let geo_file = StoredGeoFile {
            name: name.to_string(),
            file_type: file_type.to_string(),
            source_url,
            local_path: rel_path,
            size_bytes: data.len() as u64,
            updated_at: now,
            sha256: Some(sha256),
        };

        self.data.geo_files.retain(|f| !f.name.eq_ignore_ascii_case(name));
        self.data.geo_files.push(geo_file.clone());
        self.save()?;

        Ok(geo_file)
    }

    pub fn remove_geo_file(&mut self, name: &str) -> Result<(), String> {
        let init_len = self.data.geo_files.len();
        self.data.geo_files.retain(|f| !f.name.eq_ignore_ascii_case(name));
        if self.data.geo_files.len() == init_len {
            return Err(format!("Гео-файл '{}' не найден в реестре хранилища", name));
        }

        let file_path = self.get_geo_dir().join(name);
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
        }

        self.save()
    }

    // --- Резервное копирование (Backup & Recovery) ---

    pub fn create_backup(
        &self,
        name: &str,
        dest_dir: &std::path::Path,
        include_configs: bool,
        include_routes: bool,
        include_settings: bool,
    ) -> Result<PathBuf, String> {
        fs::create_dir_all(dest_dir)
            .map_err(|e| format!("Не удалось создать директорию бэкапов '{}': {}", dest_dir.display(), e))?;

        let safe_name = if name.ends_with(".json") {
            name.to_string()
        } else {
            format!("{}.json", name)
        };
        let backup_file = dest_dir.join(&safe_name);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let meta = BackupMeta {
            name: name.to_string(),
            created_at: now,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            includes_configs: include_configs,
            includes_routes: include_routes,
            includes_settings: include_settings,
        };

        let backup_data = BackupData {
            meta,
            active_picked: if include_configs { self.data.active_picked.clone() } else { None },
            last_run: if include_configs { self.data.last_run.clone() } else { None },
            groups: if include_configs { Some(self.data.groups.clone()) } else { None },
            rules: if include_routes { Some(self.data.rules.clone()) } else { None },
            settings: if include_settings { Some(self.data.settings.clone()) } else { None },
            geo_files: if include_settings || include_configs { Some(self.data.geo_files.clone()) } else { None },
        };

        let json_str = serde_json::to_string_pretty(&backup_data)
            .map_err(|e| format!("Ошибка сериализации бэкапа: {}", e))?;

        fs::write(&backup_file, json_str)
            .map_err(|e| format!("Ошибка записи файла бэкапа '{}': {}", backup_file.display(), e))?;

        Ok(backup_file)
    }

    pub fn inspect_backup(
        &self,
        backup_path: &std::path::Path,
        apply_configs: bool,
        apply_routes: bool,
        apply_settings: bool,
        mode: RecoveryMode,
    ) -> Result<RecoveryPreview, String> {
        if !backup_path.exists() {
            return Err(format!("Файл бэкапа не найден: {}", backup_path.display()));
        }

        let content = fs::read_to_string(backup_path)
            .map_err(|e| format!("Ошибка чтения файла бэкапа '{}': {}", backup_path.display(), e))?;

        let backup: BackupData = serde_json::from_str(&content)
            .map_err(|e| format!("Некорректный формат файла бэкапа: {}", e))?;

        let mut groups_preview = Vec::new();
        if apply_configs {
            if let Some(ref b_groups) = backup.groups {
                for bg in b_groups {
                    let action_desc = match mode {
                        RecoveryMode::Hard => "ПОЛНАЯ ЗАМЕНА".to_string(),
                        RecoveryMode::Merge => {
                            if let Some(cur_g) = self.get_group(&bg.name) {
                                let mut new_p = 0;
                                let mut upd_p = 0;
                                for p in &bg.profiles {
                                    if cur_g.profiles.iter().any(|cp| cp.name == p.name) {
                                        upd_p += 1;
                                    } else {
                                        new_p += 1;
                                    }
                                }
                                format!("СЛИЯНИЕ (+{} новых, {} обновятся)", new_p, upd_p)
                            } else {
                                "НОВАЯ ГРУППА".to_string()
                            }
                        }
                    };
                    groups_preview.push(GroupPreviewItem {
                        name: bg.name.clone(),
                        profiles_count: bg.profiles.len(),
                        action_desc,
                    });
                }
            }
        }

        let mut rules_preview = Vec::new();
        if apply_routes {
            if let Some(ref b_rules) = backup.rules {
                for br in b_rules {
                    let action_desc = match mode {
                        RecoveryMode::Hard => "ПОЛНАЯ ЗАМЕНА".to_string(),
                        RecoveryMode::Merge => {
                            if self.data.rules.iter().any(|cr| cr.name == br.name) {
                                "ОБНОВЛЕНИЕ".to_string()
                            } else {
                                "НОВОЕ ПРАВИЛО".to_string()
                            }
                        }
                    };
                    rules_preview.push(RulePreviewItem {
                        name: br.name.clone(),
                        action: br.action.clone(),
                        action_desc,
                    });
                }
            }
        }

        let mut settings_diffs = Vec::new();
        if apply_settings {
            if let Some(ref bs) = backup.settings {
                let cs = &self.data.settings;
                if cs.theme != bs.theme {
                    settings_diffs.push(format!("theme: '{}' -> '{}'", cs.theme, bs.theme));
                }
                if cs.mixed_port != bs.mixed_port {
                    settings_diffs.push(format!("mixed_port: {} -> {}", cs.mixed_port, bs.mixed_port));
                }
                if cs.clash_api_port != bs.clash_api_port {
                    settings_diffs.push(format!("clash_api_port: {} -> {}", cs.clash_api_port, bs.clash_api_port));
                }
                if cs.direct_dns != bs.direct_dns {
                    settings_diffs.push(format!("direct_dns: '{}' -> '{}'", cs.direct_dns, bs.direct_dns));
                }
                if cs.proxy_dns != bs.proxy_dns {
                    settings_diffs.push(format!("proxy_dns: '{}' -> '{}'", cs.proxy_dns, bs.proxy_dns));
                }
                if cs.connection_mode != bs.connection_mode {
                    settings_diffs.push(format!("connection_mode: '{}' -> '{}'", cs.connection_mode, bs.connection_mode));
                }
                if cs.log_level != bs.log_level {
                    settings_diffs.push(format!("log_level: '{}' -> '{}'", cs.log_level, bs.log_level));
                }
                if cs.max_concurrency != bs.max_concurrency {
                    settings_diffs.push(format!("max_concurrency: {} -> {}", cs.max_concurrency, bs.max_concurrency));
                }
                if cs.income_local_port != bs.income_local_port {
                    settings_diffs.push(format!("income_local_port: {} -> {}", cs.income_local_port, bs.income_local_port));
                }
                if cs.income_access_from_lan != bs.income_access_from_lan {
                    settings_diffs.push(format!("income_access_from_lan: {} -> {}", cs.income_access_from_lan, bs.income_access_from_lan));
                }
                if cs.routing_rules_source != bs.routing_rules_source {
                    settings_diffs.push(format!("routing_rules_source: '{}' -> '{}'", cs.routing_rules_source, bs.routing_rules_source));
                }
                if cs.routing_unknown_tun_traffic != bs.routing_unknown_tun_traffic {
                    settings_diffs.push(format!("routing_unknown_tun_traffic: '{}' -> '{}'", cs.routing_unknown_tun_traffic, bs.routing_unknown_tun_traffic));
                }
                if cs.routing_bypass_lan != bs.routing_bypass_lan {
                    settings_diffs.push(format!("routing_bypass_lan: {} -> {}", cs.routing_bypass_lan, bs.routing_bypass_lan));
                }
            }
        }

        let geo_files_count = backup.geo_files.as_ref().map(|g| g.len()).unwrap_or(0);

        Ok(RecoveryPreview {
            backup_name: backup.meta.name,
            backup_version: backup.meta.app_version,
            mode,
            apply_configs,
            apply_routes,
            apply_settings,
            groups: groups_preview,
            rules: rules_preview,
            settings_diffs,
            geo_files_count,
        })
    }

    pub fn restore_backup(
        &mut self,
        backup_path: &std::path::Path,
        apply_configs: bool,
        apply_routes: bool,
        apply_settings: bool,
        mode: RecoveryMode,
    ) -> Result<BackupRestoreReport, String> {
        if !backup_path.exists() {
            return Err(format!("Файл бэкапа не найден: {}", backup_path.display()));
        }

        let content = fs::read_to_string(backup_path)
            .map_err(|e| format!("Ошибка чтения файла бэкапа '{}': {}", backup_path.display(), e))?;

        let backup: BackupData = serde_json::from_str(&content)
            .map_err(|e| format!("Некорректный формат файла бэкапа: {}", e))?;

        let mut report = BackupRestoreReport::default();

        if apply_configs {
            if let Some(groups) = backup.groups {
                let p_count: usize = groups.iter().map(|g| g.profiles.len()).sum();
                report.groups_count = groups.len();
                report.profiles_count = p_count;

                match mode {
                    RecoveryMode::Hard => {
                        self.data.groups = groups;
                        if let Some(picked) = backup.active_picked {
                            self.data.active_picked = Some(picked);
                        }
                        if let Some(last) = backup.last_run {
                            self.data.last_run = Some(last);
                        }
                    }
                    RecoveryMode::Merge => {
                        for bg in groups {
                            if let Some(cur_g) = self.get_group_mut(&bg.name) {
                                for bp in bg.profiles {
                                    if let Some(pos) = cur_g.profiles.iter().position(|cp| cp.name == bp.name) {
                                        cur_g.profiles[pos] = bp;
                                    } else {
                                        cur_g.profiles.push(bp);
                                    }
                                }
                            } else {
                                self.data.groups.push(bg);
                            }
                        }
                        if self.data.active_picked.is_none() {
                            self.data.active_picked = backup.active_picked;
                        }
                        if self.data.last_run.is_none() {
                            self.data.last_run = backup.last_run;
                        }
                    }
                }
                report.configs_restored = true;
            }
        }

        if apply_routes {
            if let Some(rules) = backup.rules {
                report.rules_count = rules.len();
                match mode {
                    RecoveryMode::Hard => {
                        self.data.rules = rules;
                    }
                    RecoveryMode::Merge => {
                        for br in rules {
                            if let Some(pos) = self.data.rules.iter().position(|cr| cr.name == br.name) {
                                self.data.rules[pos] = br;
                            } else {
                                self.data.rules.push(br);
                            }
                        }
                    }
                }
                report.routes_restored = true;
            }
        }

        if apply_settings {
            if let Some(settings) = backup.settings {
                self.data.settings = settings;
                report.settings_restored = true;
            }
            if let Some(geo) = backup.geo_files {
                match mode {
                    RecoveryMode::Hard => {
                        self.data.geo_files = geo;
                    }
                    RecoveryMode::Merge => {
                        for gf in geo {
                            if !self.data.geo_files.iter().any(|cg| cg.name == gf.name) {
                                self.data.geo_files.push(gf);
                            }
                        }
                    }
                }
            }
        }

        self.save()?;

        Ok(report)
    }

    /// Группа WARP (AWG + MASQUE)
    pub fn build_warp_asset_group() -> StoredGroup {
        let mut grp = StoredGroup::new_manual("WARP");
        grp.profiles.push(StoredProfile {
            id: "warp_awg".to_string(),
            name: "Cloudflare WARP (AWG)".to_string(),
            protocol: ProtocolType::Wireguard,
            server: "162.159.192.1".to_string(),
            server_port: 2408,
            settings: serde_json::json!({
                "type": "wireguard",
                "tag": "warp-awg",
                "server": "162.159.192.1",
                "server_port": 2408,
                "local_address": ["172.16.0.2/32", "2606:4700:110:8752:629b:ff82:48f6:fb52/128"],
                "private_key": "WARP_PRIVATE_KEY_PLACEHOLDER",
                "peer_public_key": "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=",
                "reserved": [0, 0, 0],
                "mtu": 1280,
                "amnezia_wg": {
                    "jc": 4,
                    "jmin": 40,
                    "jmax": 70,
                    "s1": 0,
                    "s2": 0,
                    "h1": 1,
                    "h2": 2,
                    "h3": 3,
                    "h4": 4
                }
            }),
            tag: "warp-awg".to_string(),
            last_ping_ms: None,
        });

        grp.profiles.push(StoredProfile {
            id: "warp_masque".to_string(),
            name: "Cloudflare WARP (MASQUE)".to_string(),
            protocol: ProtocolType::Vless,
            server: "engage.cloudflareclient.com".to_string(),
            server_port: 443,
            settings: serde_json::json!({
                "type": "vless",
                "tag": "warp-masque",
                "server": "engage.cloudflareclient.com",
                "server_port": 443,
                "uuid": "00000000-0000-0000-0000-000000000000",
                "flow": "",
                "tls": {
                    "enabled": true,
                    "server_name": "engage.cloudflareclient.com",
                    "utls": {
                        "enabled": true,
                        "fingerprint": "chrome"
                    }
                },
                "transport": {
                    "type": "masque",
                    "path": "/masque"
                }
            }),
            tag: "warp-masque".to_string(),
            last_ping_ms: None,
        });

        grp
    }

    /// Группа Goida Group (Free sub)
    pub fn build_goida_asset_group() -> StoredGroup {
        let mut grp = StoredGroup::new_subscription(
            "Goida Group",
            "https://raw.githubusercontent.com/barry-far/V2ray-Configs/main/Sub1.txt",
        );
        grp.auto_update_minutes = 720;

        grp.profiles.push(StoredProfile {
            id: "goida_freedom".to_string(),
            name: "🇷🇺 GOIDA Freedom (Bypass)".to_string(),
            protocol: ProtocolType::Direct,
            server: "127.0.0.1".to_string(),
            server_port: 0,
            settings: serde_json::json!({ "type": "direct", "tag": "goida-direct" }),
            tag: "goida-direct".to_string(),
            last_ping_ms: None,
        });

        grp.profiles.push(StoredProfile {
            id: "goida_cloudflare".to_string(),
            name: "🌐 GOIDA Cloudflare Fallback".to_string(),
            protocol: ProtocolType::Vless,
            server: "104.16.132.229".to_string(),
            server_port: 443,
            settings: serde_json::json!({
                "type": "vless",
                "tag": "goida-cf",
                "server": "104.16.132.229",
                "server_port": 443,
                "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "tls": {
                    "enabled": true,
                    "server_name": "cloudflare.com"
                }
            }),
            tag: "goida-cf".to_string(),
            last_ping_ms: None,
        });

        grp
    }

    /// Установка встроенных ассетов в LocalStorage (режим merge)
    pub fn install_builtin_assets(&mut self) -> Result<Vec<String>, String> {
        let warp = Self::build_warp_asset_group();
        let goida = Self::build_goida_asset_group();

        let mut installed = Vec::new();

        if let Some(existing) = self.get_group_mut(&warp.name) {
            let mut added = 0;
            for p in warp.profiles {
                if !existing.profiles.iter().any(|ep| ep.name == p.name) {
                    existing.profiles.push(p);
                    added += 1;
                }
            }
            installed.push(format!("Обновлена группа '{}' (+{} узлов)", warp.name, added));
        } else {
            let count = warp.profiles.len();
            self.data.groups.push(warp);
            installed.push(format!("Установлена группа 'WARP' ({} узла: AWG + MASQUE)", count));
        }

        if let Some(existing) = self.get_group_mut(&goida.name) {
            let mut added = 0;
            for p in goida.profiles {
                if !existing.profiles.iter().any(|ep| ep.name == p.name) {
                    existing.profiles.push(p);
                    added += 1;
                }
            }
            installed.push(format!("Обновлена группа '{}' (+{} узлов)", goida.name, added));
        } else {
            let count = goida.profiles.len();
            self.data.groups.push(goida);
            installed.push(format!("Установлена группа 'Goida Group' (Free Sub, {} стартовых узла)", count));
        }

        self.save()?;
        Ok(installed)
    }
}
