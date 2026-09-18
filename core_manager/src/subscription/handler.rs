//! HTTP-хендлер загрузки подписок, распаковки Base64 и автоматического обновления профилей.

use crate::parser::parse_config;

use crate::storage::{ConfigStore, StoredGroup, StoredProfile};
use crate::subscription::fingerprint::FingerprintBuilder;
use crate::subscription::userinfo::SubscriptionUserInfo;
use base64::engine::general_purpose::{STANDARD, URL_SAFE};
use base64::Engine;
use reqwest::header::HeaderMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::collections::HashMap;

/// Результат успешной загрузки и парсинга подписки
#[derive(Debug, Clone)]
pub struct SubscriptionFetchResult {
    pub group_name: String,
    pub profiles: Vec<StoredProfile>,
    pub userinfo: Option<SubscriptionUserInfo>,
    pub profile_title: Option<String>,
    pub raw_content_len: usize,
    pub total_count: usize,
    pub protocol_counts: HashMap<String, usize>,
}

impl SubscriptionFetchResult {
    pub fn format_protocol_stats(&self) -> String {
        let mut parts = Vec::new();
        let mut sorted_keys: Vec<_> = self.protocol_counts.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            parts.push(format!("{}: {}", k.to_uppercase(), self.protocol_counts[k]));
        }
        if parts.is_empty() {
            "нет узлов".to_string()
        } else {
            parts.join(", ")
        }
    }
}

pub struct SubscriptionHandler;

impl SubscriptionHandler {
    const DEFAULT_TIMEOUT_SECS: u64 = 30;

    /// Загружает и парсит удаленную подписку для заданной группы
    pub async fn fetch_and_parse(group: &StoredGroup) -> Result<SubscriptionFetchResult, String> {
        let url = group
            .subscription_url
            .as_deref()
            .ok_or_else(|| format!("У группы '{}' не задан URL подписки", group.name))?;

        if url.trim().is_empty() {
            return Err(format!("У группы '{}' пустой URL подписки", group.name));
        }

        // 1. Инициализируем HTTP-клиент (с поддержкой front_proxy)
        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(Self::DEFAULT_TIMEOUT_SECS))
            .redirect(reqwest::redirect::Policy::limited(10));

        if let Some(proxy_addr) = &group.front_proxy {
            if !proxy_addr.trim().is_empty() {
                let proxy = reqwest::Proxy::all(proxy_addr)
                    .map_err(|e| format!("Некорректный front_proxy '{}': {}", proxy_addr, e))?;
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder
            .build()
            .map_err(|e| format!("Ошибка создания HTTP клиента: {}", e))?;

        // 2. Формируем фингерпринт заголовков
        let fingerprint = FingerprintBuilder::build(&group.client_imitation, group.hwid);

        let mut req = client.get(url).header("User-Agent", &fingerprint.user_agent);
        for (name, val) in &fingerprint.headers {
            req = req.header(name.as_str(), val.as_str());
        }

        // 3. Отправляем запрос
        let response = req
            .send()
            .await
            .map_err(|e| format!("Ошибка сетевого запроса к подписке: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        // 4. Проверяем специфичные заголовки ошибок HWID
        Self::check_hwid_error_headers(&headers)?;

        if !status.is_success() {
            return Err(format!(
                "Сервер подписки вернул ошибку HTTP {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            ));
        }

        // 5. Извлекаем метаинформацию (Userinfo и Название подписки)
        let userinfo = headers
            .get("subscription-userinfo")
            .and_then(|h| h.to_str().ok())
            .and_then(SubscriptionUserInfo::parse);

        let profile_title = headers
            .get("profile-title")
            .or_else(|| headers.get("content-disposition"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        // 6. Получаем тело ответа
        let raw_body = response
            .text()
            .await
            .map_err(|e| format!("Ошибка чтения ответа подписки: {}", e))?;

        let raw_len = raw_body.len();

        // 7. Распаковка содержимого: проверяем Base64 или исходный формат
        let content_to_parse = Self::decode_payload_if_base64(&raw_body);

        // 8. Парсим ноды универсальным парсером
        let parse_res = parse_config(&content_to_parse);
        if parse_res.profiles.is_empty() {
            return Err("В ответе подписки не найдено поддерживаемых конфигураций узлов".to_string());
        }

        // 9. Преобразуем ProfileConfig в StoredProfile и считаем статистику
        let mut protocol_counts = HashMap::new();
        let mut profiles = Vec::with_capacity(parse_res.profiles.len());

        for p in parse_res.profiles {
            let proto_name = p.protocol.to_string();
            *protocol_counts.entry(proto_name).or_insert(0) += 1;
            profiles.push(p.into());
        }

        let total_count = profiles.len();

        Ok(SubscriptionFetchResult {
            group_name: group.name.clone(),
            profiles,
            userinfo,
            profile_title,
            raw_content_len: raw_len,
            total_count,
            protocol_counts,
        })
    }

    /// Проверяет заголовки на наличие флагов ограничения HWID
    fn check_hwid_error_headers(headers: &HeaderMap) -> Result<(), String> {
        if let Some(h) = headers.get("x-hwid-not-supported") {
            if let Ok(v) = h.to_str() {
                if v.eq_ignore_ascii_case("true") {
                    return Err("Сервер подписки отклонил запрос: HWID не поддерживается данным провайдером".to_string());
                }
            }
        }

        if let Some(h) = headers.get("x-hwid-max-devices-reached")
            .or_else(|| headers.get("x-hwid-limit"))
        {
            if let Ok(v) = h.to_str() {
                if v.eq_ignore_ascii_case("true") {
                    return Err("Сервер подписки отклонил запрос: достигнут максимальный лимит устройств для данного HWID".to_string());
                }
            }
        }

        Ok(())
    }

    /// Проверяет, является ли строка Base64 (стандартной или url-safe), и раскодирует её
    pub fn decode_payload_if_base64(raw: &str) -> String {
        let trimmed = raw.trim();

        // Если это JSON или YAML документ — декодировать не нужно
        if trimmed.starts_with('{') || trimmed.starts_with("proxies:") || trimmed.starts_with("outbounds:") {
            return trimmed.to_string();
        }

        // Если это явный список ссылок (каждая строка начинается с протокола)
        if trimmed.starts_with("vless://")
            || trimmed.starts_with("vmess://")
            || trimmed.starts_with("ss://")
            || trimmed.starts_with("trojan://")
            || trimmed.starts_with("hy2://")
            || trimmed.starts_with("hysteria2://")
        {
            return trimmed.to_string();
        }

        // Очищаем от пробелов и переносов строк внутри base64
        let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();

        // Пробуем стандартный base64
        if let Ok(bytes) = STANDARD.decode(&cleaned) {
            if let Ok(decoded_str) = String::from_utf8(bytes) {
                if Self::contains_proxy_links(&decoded_str) {
                    return decoded_str;
                }
            }
        }

        // Пробуем URL-safe base64
        if let Ok(bytes) = URL_SAFE.decode(&cleaned) {
            if let Ok(decoded_str) = String::from_utf8(bytes) {
                if Self::contains_proxy_links(&decoded_str) {
                    return decoded_str;
                }
            }
        }

        // Fallback: возвращаем как есть
        trimmed.to_string()
    }

    fn contains_proxy_links(s: &str) -> bool {
        s.contains("://")
            || s.contains("proxies:")
            || s.contains("outbounds:")
            || s.contains("\"server\"")
    }

    /// Выполняет обновление подписки для указанной группы и сохраняет результат в хранилище
    pub async fn update_group(store: &mut ConfigStore, group_name: &str) -> Result<SubscriptionFetchResult, String> {
        let group = store
            .get_group(group_name)
            .ok_or_else(|| format!("Группа '{}' не найдена", group_name))?
            .clone();

        if group.group_type != "sub" {
            return Err(format!("Группа '{}' не является подпиской (тип: {})", group_name, group.group_type));
        }

        let fetch_res = Self::fetch_and_parse(&group).await?;

        // Обновляем ноды в хранилище
        let target_group = store
            .get_group_mut(group_name)
            .ok_or_else(|| format!("Группа '{}' не найдена", group_name))?;

        target_group.profiles = fetch_res.profiles.clone();
        target_group.subscription_userinfo = fetch_res.userinfo.clone();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        target_group.last_updated_at = Some(now);
        target_group.error_message = None;

        store.save()?;

        Ok(fetch_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64_links() {
        let plain_links = "vless://e9a0f02c-5541-4cf1-8848-0d12e6981881@94.130.1.2:443?security=tls#Node1\ntrojan://mypassword@1.2.3.4:443#Node2";
        let encoded = STANDARD.encode(plain_links);

        let decoded = SubscriptionHandler::decode_payload_if_base64(&encoded);
        assert_eq!(decoded, plain_links);

        let res = parse_config(&decoded);
        assert_eq!(res.profiles.len(), 2);
        assert_eq!(res.profiles[0].name, "Node1");
        assert_eq!(res.profiles[1].name, "Node2");
    }

    #[test]
    fn test_decode_already_plain_yaml_or_json() {
        let yaml = "proxies:\n  - name: test\n    type: ss\n    server: 1.1.1.1\n    port: 8388\n    cipher: aes-128-gcm\n    password: pwd";
        let decoded = SubscriptionHandler::decode_payload_if_base64(yaml);
        assert_eq!(decoded, yaml);
    }
}
