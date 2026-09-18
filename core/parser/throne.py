"""
Парсер конфигураций формата Throne (ссылки throne:// и JSON).
"""

import json
import urllib.parse
from typing import Optional

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class ThroneParser(BaseParser):
    """Парсер для формата Throne."""

    format_name: str = "throne"

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает ссылки формата throne:// или JSON Throne.

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
                servers = data.get("servers", [])
                for srv in servers:
                    if isinstance(srv, dict):
                        result.profiles.append(
                            ProfileConfig(
                                name=srv.get("name", "Throne Proxy"),
                                protocol=ProtocolType(srv.get("protocol", "vless")),
                                server=srv.get("server", "127.0.0.1"),
                                server_port=int(srv.get("port", 443)),
                                settings=srv.get("params", {}),
                            )
                        )
                return result
            except json.JSONDecodeError as exc:
                result.errors.append(f"Ошибка парсинга Throne JSON: {exc}")
                return result

        lines = [line.strip() for line in content.splitlines() if line.strip()]
        for line in lines:
            if line.startswith("throne://"):
                profile = self._parse_throne_link(line)
                if profile:
                    result.profiles.append(profile)

        return result

    @log_call
    def _parse_throne_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит отдельную throne:// ссылку."""
        parsed = urllib.parse.urlparse(uri)
        name = urllib.parse.unquote(parsed.fragment) or "Throne Node"
        params = urllib.parse.parse_qs(parsed.query)

        server = parsed.hostname or "127.0.0.1"
        port = parsed.port or 443

        protocol_str = params.get("mode", ["vless"])[0]
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
