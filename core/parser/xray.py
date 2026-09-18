"""
Парсер конфигураций формата Xray (JSON схема с REALITY, Vision и Mux).
"""

import json
from typing import Any, Dict, List

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class XrayParser(BaseParser):
    """Парсер для Xray JSON конфигураций."""

    format_name: str = "xray"

    PROTOCOL_MAP = {
        "vless": ProtocolType.VLESS,
        "vmess": ProtocolType.VMESS,
        "trojan": ProtocolType.TROJAN,
        "shadowsocks": ProtocolType.SHADOWSOCKS,
        "wireguard": ProtocolType.WIREGUARD,
        "freedom": ProtocolType.DIRECT,
        "blackhole": ProtocolType.BLOCK,
    }

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает JSON-конфигурацию Xray.

        Args:
            content: Текст JSON Xray.

        Returns:
            ParseResult: Результат парсинга.
        """
        result = ParseResult(format_name=self.format_name)

        try:
            data = json.loads(content)
        except json.JSONDecodeError as exc:
            result.errors.append(f"Ошибка синтаксиса Xray JSON: {exc}")
            return result

        if not isinstance(data, dict):
            result.errors.append("Корневой элемент Xray JSON должен быть словарем.")
            return result

        result.raw_config = data
        outbounds = data.get("outbounds", [])

        if not isinstance(outbounds, list):
            result.errors.append("Секция 'outbounds' Xray должна быть списком.")
            return result

        for item in outbounds:
            if not isinstance(item, dict):
                continue

            protocol_str = item.get("protocol", "").lower()
            tag = item.get("tag", "xray-outbound")

            if protocol_str in ("freedom", "blackhole", "dns"):
                continue

            protocol = self.PROTOCOL_MAP.get(protocol_str, ProtocolType.VLESS)
            settings = item.get("settings", {})
            stream_settings = item.get("streamSettings", {})

            # Вынимаем хост и порт
            server = "127.0.0.1"
            port = 443

            vnext = settings.get("vnext", [])
            servers = settings.get("servers", [])
            if vnext and isinstance(vnext, list):
                server = vnext[0].get("address", server)
                port = int(vnext[0].get("port", port))
            elif servers and isinstance(servers, list):
                server = servers[0].get("address", server)
                port = int(servers[0].get("port", port))

            profile = ProfileConfig(
                name=tag,
                protocol=protocol,
                server=server,
                server_port=port,
                settings={
                    "xray_settings": settings,
                    "stream_settings": stream_settings,
                    "mux": item.get("mux", {}),
                },
                tag=tag,
            )
            result.profiles.append(profile)

        return result
