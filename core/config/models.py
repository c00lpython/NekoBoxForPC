"""
Строгие доменные модели конфигурации NekoBoxPlusForPC.
Описывают профили, группы, маршрутизацию, DNS, TUN и настройки.
"""

from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional
import uuid


class ProtocolType(Enum):
    SHADOWSOCKS = "shadowsocks"
    VMESS = "vmess"
    VLESS = "vless"
    TROJAN = "trojan"
    HYSTERIA2 = "hysteria2"
    WIREGUARD = "wireguard"
    AMNEZIAWG = "amneziawg"
    TUIC = "tuic"
    SSH = "ssh"
    BYEDPI = "byedpi"
    DIRECT = "direct"
    BLOCK = "block"
    DNS = "dns"


class TUNStack(Enum):
    SYSTEM = "system"
    GVISOR = "gvisor"
    MIXED = "mixed"


class RoutingMode(Enum):
    GLOBAL = "global"
    RULE = "rule"
    DIRECT = "direct"


@dataclass
class ProfileConfig:
    """Модель отдельного профиля прокси-сервера."""

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    name: str = "New Profile"
    protocol: ProtocolType = ProtocolType.VLESS
    server: str = "127.0.0.1"
    server_port: int = 443
    settings: Dict[str, Any] = field(default_factory=dict)
    tag: str = ""
    group_id: Optional[str] = None
    latency_ms: Optional[float] = None
    upload_bytes: int = 0
    download_bytes: int = 0
    extra_byedpi_preset: Optional[str] = None

    def __post_init__(self):
        if not self.tag:
            self.tag = f"proxy-{self.id[:8]}"


@dataclass
class GroupConfig:
    """Группа профилей или подписка."""

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    name: str = "Default Group"
    subscription_url: Optional[str] = None
    auto_update_minutes: int = 0
    last_updated: Optional[str] = None
    profiles: List[ProfileConfig] = field(default_factory=list)


@dataclass
class RouteRule:
    """Правило маршрутизации трафика."""

    outbound: str = "proxy"
    domains: List[str] = field(default_factory=list)
    domain_suffixes: List[str] = field(default_factory=list)
    domain_keywords: List[str] = field(default_factory=list)
    ip_cidrs: List[str] = field(default_factory=list)
    geoip: List[str] = field(default_factory=list)
    geosite: List[str] = field(default_factory=list)
    port_ranges: List[str] = field(default_factory=list)
    process_names: List[str] = field(default_factory=list)
    rule_set: List[str] = field(default_factory=list)
    network: Optional[str] = None  # tcp, udp


@dataclass
class TUNConfig:
    """Настройки виртуального сетевого адаптера TUN/Wintun."""

    enabled: bool = True
    interface_name: str = "nekobox-tun"
    inet4_address: str = "172.19.0.1/30"
    inet6_address: str = "fdfe:dcba:9876::1/126"
    auto_route: bool = True
    strict_route: bool = False
    stack: TUNStack = TUNStack.MIXED
    mtu: int = 9000
    endpoint_independent_nat: bool = True


@dataclass
class InboundSettings:
    """Настройки локальных входящих портов."""

    mixed_port: int = 2080
    socks_port: int = 2081
    http_port: int = 2082
    clash_api_port: int = 9090
    clash_secret: str = ""
    v2ray_api_port: int = 10085
    allow_lan: bool = False


@dataclass
class AppConfig:
    """Полная конфигурация приложения NekoBoxPlusForPC."""

    version: str = "1.0.0"
    routing_mode: RoutingMode = RoutingMode.RULE
    inbound: InboundSettings = field(default_factory=InboundSettings)
    tun: TUNConfig = field(default_factory=TUNConfig)
    rules: List[RouteRule] = field(default_factory=list)
    groups: List[GroupConfig] = field(default_factory=list)
    active_profile_id: Optional[str] = None
    adblock_enabled: bool = True
    masterdnsvpn_enabled: bool = True
