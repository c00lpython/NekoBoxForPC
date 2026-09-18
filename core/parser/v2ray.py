"""
Парсер конфигураций формата V2Ray (URI ссылки vmess/vless/trojan/ss и JSON схемы).
"""

import base64
import json
import urllib.parse
from typing import Any, Dict, List, Optional

from core.config.models import ProfileConfig, ProtocolType
from core.parser.base import BaseParser, ParseResult
from utils.logger import log_call


class V2RayParser(BaseParser):
    """Парсер для формата V2Ray (ссылки прокси и оригинальные V2Ray JSON конфиги)."""

    format_name: str = "v2ray"

    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает входные данные V2Ray (строку со ссылками или JSON).

        Args:
            content: Текст конфигурации или URI-ссылка.

        Returns:
            ParseResult: Результат парсинга.
        """
        content = content.strip()
        result = ParseResult(format_name=self.format_name)

        if not content:
            result.errors.append("Пустое содержимое конфигурации V2Ray.")
            return result

        # Пробуем разобрать как JSON
        if content.startswith("{") and content.endswith("}"):
            try:
                data = json.loads(content)
                return self._parse_json(data)
            except json.JSONDecodeError as exc:
                result.errors.append(f"Ошибка чтения JSON V2Ray: {exc}")
                return result

        # Разбираем построчно (на случай списка ссылок)
        lines = [line.strip() for line in content.splitlines() if line.strip()]
        for line in lines:
            try:
                profile = self._parse_uri(line)
                if profile:
                    result.profiles.append(profile)
            except Exception as exc:
                result.errors.append(f"Не удалось распарсить ссылку '{line[:30]}...': {exc}")

        return result

    @log_call
    def _parse_json(self, data: Dict[str, Any]) -> ParseResult:
        """Разбирает стандартную V2Ray JSON структуру (outbounds)."""
        result = ParseResult(format_name=self.format_name, raw_config=data)
        outbounds = data.get("outbounds", [])
        if not isinstance(outbounds, list):
            result.errors.append("Раздел 'outbounds' должен быть списком.")
            return result

        for item in outbounds:
            protocol_str = item.get("protocol", "").lower()
            tag = item.get("tag", "proxy")
            settings = item.get("settings", {})
            stream_settings = item.get("streamSettings", {})

            # Игнорируем системные outbounds (freedom, blackhole)
            if protocol_str in ("freedom", "blackhole", "dns"):
                continue

            try:
                protocol_enum = ProtocolType(protocol_str)
            except ValueError:
                protocol_enum = ProtocolType.VLESS  # fallback

            # Извлекаем хост и порт из vnext / servers
            server = "127.0.0.1"
            port = 443
            vnext = settings.get("vnext", [])
            servers = settings.get("servers", [])

            if vnext and isinstance(vnext, list):
                server = vnext[0].get("address", server)
                port = vnext[0].get("port", port)
            elif servers and isinstance(servers, list):
                server = servers[0].get("address", server)
                port = servers[0].get("port", port)

            profile = ProfileConfig(
                name=tag,
                protocol=protocol_enum,
                server=server,
                server_port=int(port),
                settings={
                    "v2ray_settings": settings,
                    "stream_settings": stream_settings,
                },
                tag=tag,
            )
            result.profiles.append(profile)

        return result

    @log_call
    def _parse_uri(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит отдельную URI ссылку (vmess://, vless://, trojan://, ss://)."""
        if uri.startswith("vmess://"):
            return self._parse_vmess_link(uri)
        elif uri.startswith("vless://"):
            return self._parse_vless_link(uri)
        elif uri.startswith("trojan://"):
            return self._parse_trojan_link(uri)
        elif uri.startswith("ss://") or uri.startswith("shadowsocks://"):
            return self._parse_ss_link(uri)
        return None

    @log_call
    def _parse_vmess_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит vmess:// ссылку вида vmess://Base64(JSON)."""
        raw_b64 = uri[8:]
        padded = raw_b64 + "=" * (-len(raw_b64) % 4)
        decoded = base64.b64decode(padded).decode("utf-8", errors="ignore")
        data = json.loads(decoded)

        name = data.get("ps", "VMess Proxy")
        server = data.get("add", "127.0.0.1")
        port = int(data.get("port", 443))
        uuid_str = data.get("id", "")

        return ProfileConfig(
            name=name,
            protocol=ProtocolType.VMESS,
            server=server,
            server_port=port,
            settings={
                "uuid": uuid_str,
                "alter_id": int(data.get("aid", 0)),
                "security": data.get("scy", "auto"),
                "network": data.get("net", "tcp"),
                "type": data.get("type", "none"),
                "host": data.get("host", ""),
                "path": data.get("path", ""),
                "tls": data.get("tls", ""),
                "sni": data.get("sni", ""),
            },
        )

    @log_call
    def _parse_vless_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит vless:// ссылку вида vless://uuid@host:port?query#name."""
        parsed = urllib.parse.urlparse(uri)
        uuid_str = parsed.username or ""
        server = parsed.hostname or "127.0.0.1"
        port = parsed.port or 443
        name = urllib.parse.unquote(parsed.fragment) or f"VLESS {server}"

        params = urllib.parse.parse_qs(parsed.query)
        settings = {k: v[0] for k, v in params.items()}
        settings["uuid"] = uuid_str

        return ProfileConfig(
            name=name,
            protocol=ProtocolType.VLESS,
            server=server,
            server_port=port,
            settings=settings,
        )

    @log_call
    def _parse_trojan_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит trojan:// ссылку вида trojan://password@host:port?query#name."""
        parsed = urllib.parse.urlparse(uri)
        password = parsed.username or ""
        server = parsed.hostname or "127.0.0.1"
        port = parsed.port or 443
        name = urllib.parse.unquote(parsed.fragment) or f"Trojan {server}"

        params = urllib.parse.parse_qs(parsed.query)
        settings = {k: v[0] for k, v in params.items()}
        settings["password"] = password

        return ProfileConfig(
            name=name,
            protocol=ProtocolType.TROJAN,
            server=server,
            server_port=port,
            settings=settings,
        )

    @log_call
    def _parse_ss_link(self, uri: str) -> Optional[ProfileConfig]:
        """Парсит ss:// ссылку (SIP002 или классическую)."""
        clean_uri = uri.replace("shadowsocks://", "ss://")
        parsed = urllib.parse.urlparse(clean_uri)
        name = urllib.parse.unquote(parsed.fragment) or "Shadowsocks Proxy"

        # SIP002 формат или закодированный userinfo
        if "@" in parsed.netloc:
            userinfo, host_port = parsed.netloc.rsplit("@", 1)
            if ":" in host_port:
                server, port_str = host_port.split(":", 1)
                port = int(port_str)
            else:
                server, port = host_port, 8388

            # userinfo может быть base64Encoded(method:pass)
            if ":" in userinfo:
                method, password = userinfo.split(":", 1)
            else:
                padded = userinfo + "=" * (-len(userinfo) % 4)
                decoded = base64.b64decode(padded).decode("utf-8", errors="ignore")
                method, password = decoded.split(":", 1) if ":" in decoded else ("aes-256-gcm", decoded)
        else:
            # Весь netloc может быть закодирован в base64
            padded = parsed.netloc + "=" * (-len(parsed.netloc) % 4)
            decoded = base64.b64decode(padded).decode("utf-8", errors="ignore")
            # decoded: method:password@host:port
            userinfo, host_port = decoded.rsplit("@", 1)
            method, password = userinfo.split(":", 1)
            server, port_str = host_port.split(":", 1)
            port = int(port_str)

        return ProfileConfig(
            name=name,
            protocol=ProtocolType.SHADOWSOCKS,
            server=server,
            server_port=port,
            settings={
                "method": method,
                "password": password,
            },
        )
