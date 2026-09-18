//! Модуль конструктора профилей и групп в моментальном (JSON) и интерактивном режимах.

pub mod balancer;
pub mod config;
pub mod group;
pub mod protocols_schema;
pub mod proxychain;

pub use balancer::build_balancer_dialog;
pub use config::{
    build_profile_instant, build_profile_interactive, FieldSuggestions,
};
pub use group::{build_group_instant, build_group_interactive};
pub use protocols_schema::ProtocolsRegistry;
pub use proxychain::{build_proxychain_dialog, CandidateNode};
