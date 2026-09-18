"""
Парсер конфигураций формата Incy (ссылки incy:// и JSON/YAML).
"""

import base64
import json
import urllib.parse
from typing import Optional

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class IncyParser(BaseParser):
    """Парсер для формата Incy (Incy Proxy / LLC-INCY)."""

    format_name: str = "incy"

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает ссылки формата incy:// или JSON Incy.

        Args:
            content: Текст ссылки или конфигурации.

        Returns:
            ParseResult: Результат парсинга.
        """
        content = content.strip()
        result = ParseResult(format_name=self.format_name)

        if content.startswith("{") and content.endswith("}"):
            try:
                data = json.loads(content)
                result.raw_config = data
                nodes = data.get("nodes", [])
                for node in nodes:
                    if isinstance(node, dict):
                        result.profiles.append(
                            ProfileConfig(
                                name=node.get("name", "Incy Node"),
                                protocol=ProtocolType(node.get("type", "vless")),
                                server=node.get("host", "127.0.0.1"),
                                server_port=int(node.get("port", 443)),
                                settings=node.get("config", {}),
                            )
                        )
                return result
            except json.JSONDecodeError as exc:
                result.errors.append(f"Ошибка чтения Incy JSON: {exc}")
                return result

        lines = [line.strip() for line in content.splitlines() if line.strip()]
        for line in lines:
            if line.startswith("incy://"):
                profile = self._parse_incy_link(line)
                if profile:
                    result.profiles.append(profile)

        return result

    @log_call
    def _parse_incy_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит отдельную incy:// ссылку."""
        parsed = urllib.parse.urlparse(uri)
        name = urllib.parse.unquote(parsed.fragment) or "Incy Proxy"
        params = urllib.parse.parse_qs(parsed.query)

        server = parsed.hostname or "127.0.0.1"
        port = parsed.port or 443

        protocol_str = params.get("proto", ["vless"])[0]
        try:
            protocol = ProtocolType(protocol_str)
        except ValueError:
            protocol = ProtocolType.VLESS

        return ProfileConfig(
            name=name,
            protocol=protocol,
            server=server,
            server_port=port,
            settings={k: v[0] for k, v in params.items()},
        )
