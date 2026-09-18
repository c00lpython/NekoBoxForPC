//! Загрузчик и типизированная схема параметров протоколов из protocols.json.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const PROTOCOLS_JSON_RAW: &str = include_str!("../../assets/protocols.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
    pub default: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSchema {
    #[serde(default)]
    pub description: String,
    #[serde(rename = "_special")]
    pub special: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, PropertySchema>,
}

#[derive(Debug, Clone)]
pub struct ProtocolsRegistry {
    pub protocols: HashMap<String, ProtocolSchema>,
}

impl ProtocolsRegistry {
    pub fn load_embedded() -> Result<Self, String> {
        let protocols: HashMap<String, ProtocolSchema> = serde_json::from_str(PROTOCOLS_JSON_RAW)
            .map_err(|e| format!("Ошибка парсинга встроенного protocols.json: {}", e))?;
        Ok(Self { protocols })
    }

    pub fn get(&self, protocol: &str) -> Option<&ProtocolSchema> {
        self.protocols.get(protocol)
    }

    pub fn is_special(&self, protocol: &str) -> Option<&str> {
        self.protocols.get(protocol).and_then(|p| p.special.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_protocols() {
        let registry = ProtocolsRegistry::load_embedded().expect("Failed to load embedded protocols.json");
        assert!(registry.protocols.contains_key("vless"));
        assert!(registry.protocols.contains_key("proxychain"));
        assert!(registry.protocols.contains_key("balancer"));
        assert!(registry.protocols.contains_key("byedpi"));

        assert_eq!(registry.is_special("proxychain"), Some("chain"));
        assert_eq!(registry.is_special("balancer"), Some("balancer"));
        assert_eq!(registry.is_special("vless"), None);

        let vless = registry.get("vless").unwrap();
        assert!(vless.properties.contains_key("server"));
        assert!(vless.properties.contains_key("uuid"));
    }
}
