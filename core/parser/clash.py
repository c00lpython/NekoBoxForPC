"""
Парсер конфигураций формата Clash / Mihomo (YAML).
"""

from typing import Any, Dict, List
import yaml

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class ClashParser(BaseParser):
    """Парсер для YAML-конфигураций Clash и Mihomo."""

    format_name: str = "clash"

    # Маппинг типов Clash прокси на наш ProtocolType
    PROTOCOL_MAP = {
        "ss": ProtocolType.SHADOWSOCKS,
        "shadowsocks": ProtocolType.SHADOWSOCKS,
        "vmess": ProtocolType.VMESS,
        "vless": ProtocolType.VLESS,
        "trojan": ProtocolType.TROJAN,
        "hysteria2": ProtocolType.HYSTERIA2,
        "hy2": ProtocolType.HYSTERIA2,
        "wireguard": ProtocolType.WIREGUARD,
        "tuic": ProtocolType.TUIC,
        "ssh": ProtocolType.SSH,
        "direct": ProtocolType.DIRECT,
        "reject": ProtocolType.BLOCK,
    }

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает YAML-конфигурацию Clash.

        Args:
            content: Текст YAML-файла.

        Returns:
            ParseResult: Результат распарсенной конфигурации.
        """
        result = ParseResult(format_name=self.format_name)

        try:
            data = yaml.safe_load(content)
        except yaml.YAMLError as exc:
            result.errors.append(f"Ошибка синтаксиса YAML Clash: {exc}")
            return result

        if not isinstance(data, dict):
            result.errors.append("Корневой элемент Clash YAML должен быть словарем.")
            return result

        result.raw_config = data
        proxies = data.get("proxies", [])

        if not isinstance(proxies, list):
            result.errors.append("Секция 'proxies' должна быть списком.")
            return result

        for item in proxies:
            if not isinstance(item, dict):
                continue

            name = item.get("name", "Clash Proxy")
            raw_type = str(item.get("type", "")).lower()
            server = item.get("server", "127.0.0.1")
            port = int(item.get("port", 443))

            protocol = self.PROTOCOL_MAP.get(raw_type, ProtocolType.VLESS)

            # Переносим специфичные настройки прокси
            settings = {k: v for k, v in item.items() if k not in ("name", "type", "server", "port")}

            profile = ProfileConfig(
                name=name,
                protocol=protocol,
                server=server,
                server_port=port,
                settings=settings,
                tag=name,
            )
            result.profiles.append(profile)

        return result
