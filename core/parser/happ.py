"""
Парсер конфигураций формата Happ / Hiddify (ссылки happ://, hiddify:// и JSON).
"""

import base64
import json
import urllib.parse
from typing import Any, Dict, Optional

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class HappParser(BaseParser):
    """Парсер для формата Happ и Hiddify."""

    format_name: str = "happ"

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает ссылки формата happ:// / hiddify:// или JSON Happ.

        Args:
            content: Текст ссылки или JSON.

        Returns:
            ParseResult: Результат парсинга.
        """
        content = content.strip()
        result = ParseResult(format_name=self.format_name)

        if content.startswith("{") and content.endswith("}"):
            try:
                data = json.loads(content)
                result.raw_config = data
                profiles = data.get("profiles", [])
                for item in profiles:
                    if isinstance(item, dict):
                        result.profiles.append(
                            ProfileConfig(
                                name=item.get("name", "Happ Profile"),
                                protocol=ProtocolType(item.get("protocol", "vless")),
                                server=item.get("server", "127.0.0.1"),
                                server_port=int(item.get("port", 443)),
                                settings=item.get("settings", {}),
                            )
                        )
                return result
            except json.JSONDecodeError as exc:
                result.errors.append(f"Ошибка синтаксиса Happ JSON: {exc}")
                return result

        lines = [line.strip() for line in content.splitlines() if line.strip()]
        for line in lines:
            if line.startswith("happ://") or line.startswith("hiddify://"):
                profile = self._parse_happ_link(line)
                if profile:
                    result.profiles.append(profile)

        return result

    @log_call
    def _parse_happ_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит ссылку happ:// или hiddify://."""
        scheme_len = 7 if uri.startswith("happ://") else 10
        raw_data = uri[scheme_len:]
        
        parsed = urllib.parse.urlparse(uri)
        name = urllib.parse.unquote(parsed.fragment) or "Happ Proxy"

        # Пробуем декодировать base64 payload если параметр один
        if "?" in raw_data:
            params = urllib.parse.parse_qs(parsed.query)
            server = parsed.hostname or "127.0.0.1"
            port = parsed.port or 443
            protocol_str = params.get("type", ["vless"])[0]

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
        elif raw_data:
            try:
                padded = raw_data + "=" * (-len(raw_data) % 4)
                decoded = base64.b64decode(padded).decode("utf-8", errors="ignore")
                if decoded.startswith("{"):
                    data = json.loads(decoded)
                    return ProfileConfig(
                        name=data.get("name", name),
                        protocol=ProtocolType(data.get("protocol", "vless")),
                        server=data.get("server", "127.0.0.1"),
                        server_port=int(data.get("port", 443)),
                        settings=data.get("settings", {}),
                    )
            except Exception:
                pass

        return ProfileConfig(
            name=name,
            protocol=ProtocolType.VLESS,
            server=parsed.hostname or "127.0.0.1",
            server_port=parsed.port or 443,
        )
