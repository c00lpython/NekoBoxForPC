"""
Конвертер профилей ProfileConfig и AppConfig в итоговую JSON-конфигурацию sing-box.
"""

from typing import Any, Dict, List, Optional

from core.config.models import AppConfig, ProfileConfig, ProtocolType, RoutingMode
from utils.logger import log_call


class SingBoxConverter:
    """Конвертер внутреннего представления профилей NekoBox в sing-box JSON."""

    @classmethod
    @log_call
    def to_singbox_json(
        cls,
        profiles: List[ProfileConfig],
        app_config: Optional[AppConfig] = None,
        active_profile_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Генерирует полновластный валидный JSON-конфиг для запуска ядра sing-box.

        Args:
            profiles: Список спарсенных профилей.
            app_config: Глобальные настройки приложения (DNS, TUN, маршруты).
            active_profile_id: Идентификатор активного профиля.

        Returns:
            Dict[str, Any]: Словарь конфигурации sing-box.
        """
        if app_config is None:
            app_config = AppConfig()

        # Определяем активный профиль
        active_profile: Optional[ProfileConfig] = None
        if active_profile_id:
            for p in profiles:
                if p.id == active_profile_id:
                    active_profile = p
                    break

        if not active_profile and profiles:
            active_profile = profiles[0]

        outbounds = []

        # Формируем главный outbound для активного профиля
        if active_profile:
            main_outbound = cls._profile_to_outbound(active_profile, tag="proxy")
            outbounds.append(main_outbound)
        else:
            # Дефолтный fallback outbound
            outbounds.append({
                "type": "direct",
                "tag": "proxy",
            })

        # Добавляем стандартные outbounds
        outbounds.append({"type": "direct", "tag": "direct"})
        outbounds.append({"type": "block", "tag": "block"})
        outbounds.append({"type": "dns", "tag": "dns-out"})

        # Формируем inbounds (Mixed port + TUN)
        inbounds = [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1" if not app_config.inbound.allow_lan else "0.0.0.0",
                "listen_port": app_config.inbound.mixed_port,
                "sniff": True,
            }
        ]

        if app_config.tun.enabled:
            inbounds.append({
                "type": "tun",
                "tag": "tun-in",
                "interface_name": app_config.tun.interface_name,
                "inet4_address": [app_config.tun.inet4_address],
                "inet6_address": [app_config.tun.inet6_address] if app_config.tun.inet6_address else [],
                "auto_route": app_config.tun.auto_route,
                "strict_route": app_config.tun.strict_route,
                "stack": app_config.tun.stack.value,
                "mtu": app_config.tun.mtu,
                "sniff": True,
            })

        # Маршрутизация
        rules = []
        if app_config.routing_mode == RoutingMode.GLOBAL:
            rules.append({"outbound": "proxy"})
        elif app_config.routing_mode == RoutingMode.DIRECT:
            rules.append({"outbound": "direct"})
        else:
            # Rule mode
            for rule in app_config.rules:
                rule_dict: Dict[str, Any] = {"outbound": rule.outbound}
                if rule.domains:
                    rule_dict["domain"] = rule.domains
                if rule.domain_suffixes:
                    rule_dict["domain_suffix"] = rule.domain_suffixes
                if rule.domain_keywords:
                    rule_dict["domain_keyword"] = rule.domain_keywords
                if rule.ip_cidrs:
                    rule_dict["ip_cidr"] = rule.ip_cidrs
                if rule.geoip:
                    rule_dict["geoip"] = rule.geoip
                if rule.geosite:
                    rule_dict["geosite"] = rule.geosite
                rules.append(rule_dict)

        # Стандартные правила DNS
        rules.insert(0, {"protocol": "dns", "outbound": "dns-out"})

        config: Dict[str, Any] = {
            "log": {
                "level": "info",
                "timestamp": True,
            },
            "dns": {
                "servers": [
                    {
                        "tag": "google-dns",
                        "address": "tls://8.8.8.8",
                    },
                    {
                        "tag": "local-dns",
                        "address": "local",
                        "detour": "direct",
                    },
                ],
                "rules": [
                    {"outbound": ["any"], "server": "google-dns"},
                ],
            },
            "inbounds": inbounds,
            "outbounds": outbounds,
            "route": {
                "rules": rules,
                "auto_detect_interface": True,
            },
        }

        return config

    @classmethod
    @log_call
    def _profile_to_outbound(cls, profile: ProfileConfig, tag: str = "proxy") -> Dict[str, Any]:
        """Преобразует ProfileConfig в словарь outbound для sing-box."""
        proto = profile.protocol

        outbound: Dict[str, Any] = {
            "type": proto.value,
            "tag": tag,
            "server": profile.server,
            "server_port": profile.server_port,
        }

        settings = profile.settings or {}

        if proto == ProtocolType.VLESS:
            outbound["uuid"] = settings.get("uuid", "")
            if settings.get("flow"):
                outbound["flow"] = settings.get("flow")

            tls_enabled = settings.get("security") in ("tls", "reality") or bool(settings.get("tls"))
            if tls_enabled:
                tls_config: Dict[str, Any] = {
                    "enabled": True,
                    "server_name": settings.get("sni") or settings.get("host") or profile.server,
                }
                if settings.get("security") == "reality" or settings.get("pbk"):
                    tls_config["reality"] = {
                        "enabled": True,
                        "public_key": settings.get("pbk", ""),
                        "short_id": settings.get("sid", ""),
                    }
                outbound["tls"] = tls_config

        elif proto == ProtocolType.VMESS:
            outbound["uuid"] = settings.get("uuid", "")
            outbound["security"] = settings.get("security", "auto")

        elif proto == ProtocolType.TROJAN:
            outbound["password"] = settings.get("password", "")

        elif proto == ProtocolType.SHADOWSOCKS:
            outbound["method"] = settings.get("method", "aes-256-gcm")
            outbound["password"] = settings.get("password", "")

        elif proto == ProtocolType.HYSTERIA2:
            outbound["password"] = settings.get("password", "")

        return outbound


@log_call
def to_singbox_json(
    profiles: List[ProfileConfig],
    app_config: Optional[AppConfig] = None,
    active_profile_id: Optional[str] = None,
) -> Dict[str, Any]:
    """Утилитарная функция для конвертации в sing-box JSON."""
    return SingBoxConverter.to_singbox_json(profiles, app_config, active_profile_id)
