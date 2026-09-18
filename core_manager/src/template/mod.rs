//! Модуль генератора Ultimate-конфигурации sing-box с адресацией по путям/индексам.

use crate::models::ProfileConfig;
use serde_json::{json, Value};

/// Сегмент пути в JSON-структуре (ключ словаря или числовой индекс массива).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

/// Парсер путей вида `[outbounds][0]`, `outbounds[0]`, `[dns][servers][1]` или `route.rules.0`.
pub fn parse_path(path_str: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let text = path_str.trim();

    if text.starts_with('[') && text.contains(']') {
        // Парсинг скобочной нотации [outbounds][0][server]
        let parts = text.split(']').filter_map(|s| {
            let trimmed = s.trim_start_matches('[').trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        for p in parts {
            if let Ok(idx) = p.parse::<usize>() {
                segments.push(PathSegment::Index(idx));
            } else {
                segments.push(PathSegment::Key(p.to_string()));
            }
        }
    } else {
        // Парсинг точечной нотации outbounds.0.server
        for p in text.split('.') {
            let p_trim = p.trim();
            if p_trim.is_empty() {
                continue;
            }
            if let Ok(idx) = p_trim.parse::<usize>() {
                segments.push(PathSegment::Index(idx));
            } else {
                segments.push(PathSegment::Key(p_trim.to_string()));
            }
        }
    }

    segments
}

/// Точечная инъекция значения в JSON-дерево по разобранному пути.
/// Если промежуточных массивов или ключей нет — они создаются автоматически.
pub fn inject_at_path(
    root: &mut Value,
    path_str: &str,
    new_value: Value,
) -> Result<(), String> {
    let segments = parse_path(path_str);
    if segments.is_empty() {
        return Err("Пустой путь для инъекции".into());
    }

    let mut current = root;

    for i in 0..segments.len() - 1 {
        let seg = &segments[i];
        let next_seg = &segments[i + 1];

        match seg {
            PathSegment::Key(key) => {
                if !current.is_object() {
                    *current = Value::Object(serde_json::Map::new());
                }
                let obj = current.as_object_mut().unwrap();
                current = obj.entry(key.clone()).or_insert_with(|| {
                    match next_seg {
                        PathSegment::Index(_) => Value::Array(Vec::new()),
                        PathSegment::Key(_) => Value::Object(serde_json::Map::new()),
                    }
                });
            }
            PathSegment::Index(idx) => {
                if !current.is_array() {
                    *current = Value::Array(Vec::new());
                }
                let arr = current.as_array_mut().unwrap();
                while arr.len() <= *idx {
                    let default_val = match next_seg {
                        PathSegment::Index(_) => Value::Array(Vec::new()),
                        PathSegment::Key(_) => Value::Object(serde_json::Map::new()),
                    };
                    arr.push(default_val);
                }
                current = &mut arr[*idx];
            }
        }
    }

    // Запись в последний сегмент
    let last_seg = segments.last().unwrap();
    match last_seg {
        PathSegment::Key(key) => {
            if !current.is_object() {
                *current = Value::Object(serde_json::Map::new());
            }
            current.as_object_mut().unwrap().insert(key.clone(), new_value);
        }
        PathSegment::Index(idx) => {
            if !current.is_array() {
                *current = Value::Array(Vec::new());
            }
            let arr = current.as_array_mut().unwrap();
            while arr.len() <= *idx {
                arr.push(Value::Null);
            }
            arr[*idx] = new_value;
        }
    }

    Ok(())
}

/// Базовый Master-шаблон для ядра sing-box.
pub fn get_default_master_template() -> Value {
    json!({
        "log": {
            "level": "trace",
            "timestamp": true
        },
        "dns": {
            "servers": [
                {
                    "tag": "local-dns",
                    "type": "local"
                },
                {
                    "tag": "remote-dns",
                    "type": "https",
                    "server": "1.1.1.1",
                    "path": "/dns-query"
                },
                {
                    "tag": "direct-dns",
                    "type": "https",
                    "server": "77.88.8.8",
                    "path": "/dns-query"
                }
            ],
            "rules": [
                {
                    "clash_mode": "Direct",
                    "server": "local-dns"
                },
                {
                    "clash_mode": "Global",
                    "server": "remote-dns"
                },
                {
                    "domain_suffix": [".ru", ".su", ".xn--p1ai"],
                    "server": "direct-dns"
                }
            ],
            "final": "remote-dns",
            "strategy": "prefer_ipv4",
            "independent_cache": true
        },
        "inbounds": [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 20808
            },
            {
                "type": "tun",
                "tag": "tun-in",
                "interface_name": "nekobox-tun",
                "inet4_address": ["172.19.0.1/30"],
                "auto_route": true,
                "strict_route": false,
                "stack": "mixed",
                "mtu": 9000
            }
        ],
        "outbounds": [
            {
                "type": "direct",
                "tag": "proxy"
            },
            {
                "type": "direct",
                "tag": "direct"
            },
            {
                "type": "block",
                "tag": "block"
            }
        ],
        "route": {
            "rules": [
                {
                    "action": "sniff"
                },
                {
                    "protocol": "dns",
                    "action": "hijack-dns"
                },
                {
                    "ip_is_private": true,
                    "outbound": "direct"
                },
                {
                    "clash_mode": "Direct",
                    "outbound": "direct"
                },
                {
                    "clash_mode": "Global",
                    "outbound": "proxy"
                }
            ],
            "default_domain_resolver": "local-dns",
            "auto_detect_interface": true,
            "final": "proxy"
        },
        "experimental": {
            "clash_api": {
                "external_controller": "127.0.0.1:9090",
                "external_ui": "metacubexd",
                "secret": "",
                "default_mode": "rule"
            },
            "cache_file": {
                "enabled": true,
                "path": "cache.db"
            }
        }
    })
}

/// Строитель итоговой конфигурации ядра через инъекцию по путям.
pub struct UltimateConfigBuilder {
    template: Value,
}

impl UltimateConfigBuilder {
    pub fn new() -> Self {
        Self {
            template: get_default_master_template(),
        }
    }

    pub fn with_custom_template(template: Value) -> Self {
        Self { template }
    }

    /// Инъекция произвольного JSON по указанному пути/индексам.
    pub fn inject(&mut self, path: &str, value: Value) -> Result<&mut Self, String> {
        inject_at_path(&mut self.template, path, value)?;
        Ok(self)
    }

    fn remove_remote_dns_detour(&mut self) {
        if let Some(dns_servers) = self.template.get_mut("dns").and_then(|d| d.get_mut("servers")).and_then(|s| s.as_array_mut()) {
            if let Some(remote_dns) = dns_servers.get_mut(1).and_then(|s| s.as_object_mut()) {
                remote_dns.remove("detour");
            }
        }
    }

    fn add_direct_bypass_for_server(&mut self, srv: &str) {
        let srv = srv.trim();
        if srv.is_empty() || srv == "127.0.0.1" || srv == "localhost" {
            return;
        }

        // 1. Маршрутизация трафика к самому серверу прокси напрямую (direct)
        if let Some(rules) = self.template.get_mut("route").and_then(|r| r.get_mut("rules")).and_then(|r| r.as_array_mut()) {
            let rule_obj = if srv.parse::<std::net::IpAddr>().is_ok() {
                json!({
                    "ip_cidr": [format!("{}/32", srv)],
                    "outbound": "direct"
                })
            } else {
                json!({
                    "domain": [srv.to_string()],
                    "outbound": "direct"
                })
            };
            let insert_idx = rules
                .iter()
                .position(|r| r.get("protocol").and_then(|p| p.as_str()) == Some("dns"))
                .map(|idx| idx + 1)
                .unwrap_or(0);
            rules.insert(insert_idx, rule_obj);
        }

        // 2. Локальное DNS-разрешение домена сервера через local-dns (устраняет дедлок)
        if srv.parse::<std::net::IpAddr>().is_err() {
            if let Some(dns_rules) = self.template.get_mut("dns").and_then(|d| d.get_mut("rules")).and_then(|r| r.as_array_mut()) {
                dns_rules.insert(0, json!({
                    "domain": [srv.to_string()],
                    "server": "local-dns"
                }));
            }
        }
    }

    /// Вставляет спарсенный прокси-профиль в слот основного прокси [outbounds][0].
    pub fn inject_profile(&mut self, profile: &ProfileConfig) -> Result<&mut Self, String> {
        let outbound = crate::converter::singbox::profile_to_outbound(profile, "proxy");
        self.inject("[outbounds][0]", outbound)?;

        self.add_direct_bypass_for_server(&profile.server);

        let is_direct = profile.protocol == crate::models::ProtocolType::Direct;
        if is_direct {
            self.remove_remote_dns_detour();
        } else {
            let _ = self.inject("[dns][servers][1][detour]", json!("proxy"));
        }

        Ok(self)
    }

    /// Устанавливает прямой локальный выход (Direct Outbound).
    pub fn inject_direct(&mut self) -> Result<&mut Self, String> {
        self.inject("[outbounds][0]", json!({
            "type": "direct",
            "tag": "proxy"
        }))?;
        self.remove_remote_dns_detour();
        Ok(self)
    }

    /// Формирует последовательную цепочку прокси через механизм detour (например, vless->vless или awg->vless).
    pub fn inject_chain(&mut self, nodes: &[ProfileConfig]) -> Result<&mut Self, String> {
        if nodes.is_empty() {
            return Err("Цепочка прокси не может быть пустой".into());
        }
        if nodes.len() == 1 {
            return self.inject_profile(&nodes[0]);
        }

        for node in nodes {
            self.add_direct_bypass_for_server(&node.server);
        }

        let mut chain_outbounds = Vec::new();
        let n = nodes.len();

        for (i, node) in nodes.iter().enumerate() {
            let tag = if i == n - 1 {
                "proxy".to_string()
            } else {
                format!("chain-node-{}", i)
            };
            let mut ob = crate::converter::singbox::profile_to_outbound(node, &tag);
            if i > 0 {
                let prev_tag = format!("chain-node-{}", i - 1);
                ob["detour"] = json!(prev_tag);
            }
            chain_outbounds.push(ob);
        }

        let mut final_outbounds = chain_outbounds;
        final_outbounds.push(json!({"type": "direct", "tag": "direct"}));
        final_outbounds.push(json!({"type": "block", "tag": "block"}));

        if let Some(obj) = self.template.as_object_mut() {
            obj.insert("outbounds".to_string(), Value::Array(final_outbounds));
        }

        let _ = self.inject("[dns][servers][1][detour]", json!("proxy"));

        Ok(self)
    }

    /// Формирует балансировщик типа urltest по списку кандидатов с опциональным detour.
    pub fn inject_balancer(&mut self, tag: &str, candidates: &[ProfileConfig], detour: Option<&str>) -> Result<&mut Self, String> {
        if candidates.is_empty() {
            return Err("Список кандидатов балансировщика не может быть пустым".into());
        }

        for cand in candidates {
            self.add_direct_bypass_for_server(&cand.server);
        }

        let mut candidate_tags = Vec::new();
        let mut bal_outbounds = Vec::new();

        for (i, cand) in candidates.iter().enumerate() {
            let c_tag = format!("{}-cand-{}", tag, i);
            let mut ob = crate::converter::singbox::profile_to_outbound(cand, &c_tag);
            if let Some(d) = detour {
                ob["detour"] = json!(d);
            }
            bal_outbounds.push(ob);
            candidate_tags.push(c_tag);
        }

        let balancer_outbound = json!({
            "type": "urltest",
            "tag": tag,
            "outbounds": candidate_tags,
            "url": "https://www.gstatic.com/generate_204",
            "interval": "3m",
            "tolerance": 50
        });

        let mut final_outbounds = Vec::new();
        final_outbounds.push(balancer_outbound);
        final_outbounds.extend(bal_outbounds);
        final_outbounds.push(json!({"type": "direct", "tag": "direct"}));
        final_outbounds.push(json!({"type": "block", "tag": "block"}));

        if let Some(obj) = self.template.as_object_mut() {
            obj.insert("outbounds".to_string(), Value::Array(final_outbounds));
        }

        let _ = self.inject("[dns][servers][1][detour]", json!("proxy"));

        Ok(self)
    }

    /// Формирует цепочку с балансировщиком (например, AmneziaWG -> Group Balancer).
    pub fn inject_chain_balancer(&mut self, front_node: &ProfileConfig, balancer_candidates: &[ProfileConfig]) -> Result<&mut Self, String> {
        if balancer_candidates.is_empty() {
            return Err("Список кандидатов балансировщика не может быть пустым".into());
        }

        self.add_direct_bypass_for_server(&front_node.server);
        for cand in balancer_candidates {
            self.add_direct_bypass_for_server(&cand.server);
        }

        let front_tag = "chain-front";
        let front_ob = crate::converter::singbox::profile_to_outbound(front_node, front_tag);

        let mut candidate_tags = Vec::new();
        let mut cand_outbounds = Vec::new();

        for (i, cand) in balancer_candidates.iter().enumerate() {
            let c_tag = format!("proxy-cand-{}", i);
            let mut ob = crate::converter::singbox::profile_to_outbound(cand, &c_tag);
            ob["detour"] = json!(front_tag);
            cand_outbounds.push(ob);
            candidate_tags.push(c_tag);
        }

        let balancer_outbound = json!({
            "type": "urltest",
            "tag": "proxy",
            "outbounds": candidate_tags,
            "url": "https://www.gstatic.com/generate_204",
            "interval": "3m",
            "tolerance": 50
        });

        let mut final_outbounds = Vec::new();
        final_outbounds.push(balancer_outbound);
        final_outbounds.push(front_ob);
        final_outbounds.extend(cand_outbounds);
        final_outbounds.push(json!({"type": "direct", "tag": "direct"}));
        final_outbounds.push(json!({"type": "block", "tag": "block"}));

        if let Some(obj) = self.template.as_object_mut() {
            obj.insert("outbounds".to_string(), Value::Array(final_outbounds));
        }

        let _ = self.inject("[dns][servers][1][detour]", json!("proxy"));

        Ok(self)
    }

    /// Включает/выключает TUN-интерфейс в inbounds.
    pub fn set_tun_enabled(&mut self, enabled: bool) -> Result<&mut Self, String> {
        if !enabled {
            if let Some(inbounds) = self.template.get_mut("inbounds").and_then(|i| i.as_array_mut()) {
                inbounds.retain(|ib| ib.get("type").and_then(|t| t.as_str()) != Some("tun"));
            }
        }
        if let Some(route) = self.template.get_mut("route").and_then(|r| r.as_object_mut()) {
            route.insert("auto_detect_interface".to_string(), json!(true));
        }
        Ok(self)
    }

    /// Настраивает адрес и порт Clash API (для Metacubexd).
    pub fn set_clash_api(&mut self, listen_addr: &str, ui_dir: &str) -> Result<&mut Self, String> {
        self.inject("[experimental][clash_api][external_controller]", json!(listen_addr))?;
        self.inject("[experimental][clash_api][external_ui]", json!(ui_dir))?;
        Ok(self)
    }

    /// Настраивает уровень логирования ядра sing-box (trace, debug, info, warn, error).
    pub fn set_log_level(&mut self, level: &str) -> Result<&mut Self, String> {
        self.inject("[log][level]", json!(level))?;
        Ok(self)
    }

    /// Внедряет пользовательские правила маршрутизации и локальные пути к гео-базам.
    pub fn inject_rules_and_geo(
        &mut self,
        rules: &[crate::storage::StoredRule],
        geo_dir: Option<&std::path::Path>,
    ) -> Result<&mut Self, String> {
        if let Some(dir) = geo_dir {
            let geoip = dir.join("geoip.db");
            let geosite = dir.join("geosite.db");
            if geoip.exists() {
                let _ = self.inject("[route][geoip][path]", json!(geoip.to_string_lossy().to_string()));
            }
            if geosite.exists() {
                let _ = self.inject("[route][geosite][path]", json!(geosite.to_string_lossy().to_string()));
            }
        }

        if !rules.is_empty() {
            if let Some(route_rules) = self.template.get_mut("route").and_then(|r| r.get_mut("rules")).and_then(|r| r.as_array_mut()) {
                for rule in rules {
                    if !rule.enabled {
                        continue;
                    }
                    let mut rule_obj = serde_json::Map::new();
                    if !rule.site.is_empty() {
                        rule_obj.insert("domain".to_string(), json!(rule.site));
                    }
                    if !rule.ip.is_empty() {
                        rule_obj.insert("ip_cidr".to_string(), json!(rule.ip));
                    }
                    rule_obj.insert("outbound".to_string(), json!(rule.action));
                    route_rules.insert(0, Value::Object(rule_obj));
                }
            }
        }

        Ok(self)
    }

    pub fn build(self) -> Value {
        self.template
    }
}

impl Default for UltimateConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
