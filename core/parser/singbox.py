"""
Парсер родногo формата конфигураций sing-box (JSON).
"""

import json
from typing import Any, Dict, List

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class SingBoxParser(BaseParser):
    """Парсер для native sing-box JSON конфигураций."""

    format_name: str = "sing-box"

    TYPE_MAP = {
        "shadowsocks": ProtocolType.SHADOWSOCKS,
        "vmess": ProtocolType.VMESS,
        "vless": ProtocolType.VLESS,
        "trojan": ProtocolType.TROJAN,
        "hysteria2": ProtocolType.HYSTERIA2,
        "wireguard": ProtocolType.WIREGUARD,
        "tuic": ProtocolType.TUIC,
        "ssh": ProtocolType.SSH,
        "direct": ProtocolType.DIRECT,
        "block": ProtocolType.BLOCK,
        "dns": ProtocolType.DNS,
    }

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает JSON-конфигурацию sing-box.

        Args:
            content: Текст JSON sing-box.

        Returns:
            ParseResult: Результат парсинга.
        """
        result = ParseResult(format_name=self.format_name)

        try:
            data = json.loads(content)
        except json.JSONDecodeError as exc:
            result.errors.append(f"Ошибка синтаксиса sing-box JSON: {exc}")
            return result

        if not isinstance(data, dict):
            result.errors.append("Корневой элемент sing-box JSON должен быть словарем.")
            return result

        result.raw_config = data
        outbounds = data.get("outbounds", [])

        if not isinstance(outbounds, list):
            result.errors.append("Секция 'outbounds' должна быть списком.")
            return result

        for item in outbounds:
            if not isinstance(item, dict):
                continue

            out_type = item.get("type", "").lower()
            tag = item.get("tag", "singbox-outbound")

            # Пропускаем служебные outbounds
            if out_type in ("dns", "direct", "block"):
                continue

            server = item.get("server", "127.0.0.1")
            server_port = int(item.get("server_port", 443))

            protocol = self.TYPE_MAP.get(out_type, ProtocolType.VLESS)

            # Сохраняем остальные поля в settings
            settings = {k: v for k, v in item.items() if k not in ("type", "tag", "server", "server_port")}

            profile = ProfileConfig(
                name=tag,
                protocol=protocol,
                server=server,
                server_port=server_port,
                settings=settings,
                tag=tag,
            )
            result.profiles.append(profile)

        return result
