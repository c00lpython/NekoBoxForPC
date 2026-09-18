//! Подсистема подписок: HTTP-хендлер, имитация клиентов, генератор HWID и парсер Subscription-Userinfo.

pub mod fingerprint;
pub mod handler;
pub mod userinfo;

pub use fingerprint::{FingerprintBuilder, RequestFingerprint};
pub use handler::{SubscriptionFetchResult, SubscriptionHandler};
pub use userinfo::SubscriptionUserInfo;
