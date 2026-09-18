//! Генератор фингерпринтов HTTP-клиента и заголовков имитации приложений (SpoofApp / HWID).

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct RequestFingerprint {
    pub user_agent: String,
    pub headers: BTreeMap<String, String>,
}

pub struct FingerprintBuilder;

impl FingerprintBuilder {
    pub const UA_V2RAYN: &'static str = "v2rayN/6.23";
    pub const UA_CLASH_META: &'static str = "ClashMeta/v1.18.0";
    pub const UA_SING_BOX: &'static str = "sing-box/1.9.0";
    pub const UA_SHADOWROCKET: &'static str = "Shadowrocket/2.2.35";
    pub const UA_HAPP: &'static str = "Happ/3.26.3/Android/17839452147361875676";
    pub const UA_V2RAYTUN: &'static str = "v2raytun/android";
    pub const UA_INCY: &'static str = "INCY/3.4.3/android Dalvik/2.1.0";

    /// Генерирует детерминированный стабильный HWID для текущей машины
    pub fn generate_hwid() -> String {
        // Комбинируем имя хоста, архитектуру и ОС
        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "nbpfpc-client".to_string());
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let seed = format!("{}-{}-{}", hostname, os, arch);

        // Формируем стабильный 32-символьный hex хеш
        format!("{:032x}", Self::simple_hash(&seed))
    }

    fn simple_hash(input: &str) -> u128 {
        let mut h: u128 = 0xcbf29ce484222325;
        for b in input.bytes() {
            h ^= b as u128;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    /// Формирует User-Agent и набор заголовков для имитации целевого приложения
    pub fn build(client_imitation: &str, hwid_enabled: bool) -> RequestFingerprint {
        let mut headers = BTreeMap::new();
        let imitation_lower = client_imitation.trim().to_lowercase();

        let user_agent = match imitation_lower.as_str() {
            "clash" | "clashmeta" | "meta" => Self::UA_CLASH_META.to_string(),
            "singbox" | "sing-box" => Self::UA_SING_BOX.to_string(),
            "shadowrocket" => Self::UA_SHADOWROCKET.to_string(),
            "happ" => {
                headers.insert("X-Device-Model".to_string(), "PC-Desktop".to_string());
                headers.insert("X-Ver-Os".to_string(), "10".to_string());
                headers.insert("X-Device-Os".to_string(), "Windows".to_string());
                headers.insert("X-Device-Locale".to_string(), "ru".to_string());
                Self::UA_HAPP.to_string()
            }
            "v2raytun" => {
                headers.insert("X-App-Version".to_string(), "5.25.80".to_string());
                headers.insert("X-Device-Model".to_string(), "Desktop PC".to_string());
                headers.insert("X-Ver-Os".to_string(), "Windows 11".to_string());
                headers.insert("X-Device-Os".to_string(), "Windows".to_string());
                Self::UA_V2RAYTUN.to_string()
            }
            "incy" => {
                headers.insert("Accept".to_string(), "*/*".to_string());
                headers.insert("X-Client".to_string(), "INCY".to_string());
                headers.insert("X-App-Version".to_string(), "3.4.3".to_string());
                headers.insert("X-Device-Model".to_string(), "Desktop PC".to_string());
                headers.insert("X-Device-Os".to_string(), "Windows".to_string());
                Self::UA_INCY.to_string()
            }
            _ => {
                // v2rayN по умолчанию
                Self::UA_V2RAYN.to_string()
            }
        };

        if hwid_enabled {
            let hwid = Self::generate_hwid();
            headers.insert("X-Hwid".to_string(), hwid);
            if !headers.contains_key("X-Device-Os") {
                headers.insert("X-Device-Os".to_string(), "Windows".to_string());
            }
        }

        RequestFingerprint {
            user_agent,
            headers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2rayn_fingerprint() {
        let fp = FingerprintBuilder::build("v2rayN", false);
        assert_eq!(fp.user_agent, FingerprintBuilder::UA_V2RAYN);
        assert!(!fp.headers.contains_key("X-Hwid"));
    }

    #[test]
    fn test_clash_meta_fingerprint() {
        let fp = FingerprintBuilder::build("ClashMeta", true);
        assert_eq!(fp.user_agent, FingerprintBuilder::UA_CLASH_META);
        assert!(fp.headers.contains_key("X-Hwid"));
    }

    #[test]
    fn test_happ_fingerprint_with_hwid() {
        let fp = FingerprintBuilder::build("happ", true);
        assert_eq!(fp.user_agent, FingerprintBuilder::UA_HAPP);
        assert_eq!(fp.headers.get("X-Device-Os").unwrap(), "Windows");
        assert!(fp.headers.contains_key("X-Hwid"));
    }
}
