"""
Детектор формата конфигураций и универсальная точка входа парсинга.
Определяет 7 форматов: v2ray, clash, sing-box, xray, happ, incy, throne.
"""

import json
from typing import Dict, Type
import yaml

from core.parser.base import BaseParser, ParseResult
from core.parser.clash import ClashParser
from core.parser.happ import HappParser
from core.parser.incy import IncyParser
from core.parser.singbox import SingBoxParser
from core.parser.throne import ThroneParser
from core.parser.v2ray import V2RayParser
from core.parser.xray import XrayParser
from utils.logger import log_call


class FormatDetector:
    """Класс автоопределения формата прокси-конфигураций."""

    PARSERS: Dict[str, Type[BaseParser]] = {
        "v2ray": V2RayParser,
        "clash": ClashParser,
        "sing-box": SingBoxParser,
        "xray": XrayParser,
        "happ": HappParser,
        "incy": IncyParser,
        "throne": ThroneParser,
    }

    @classmethod
    @log_call
    def detect(cls, content: str) -> str:
        """
        Определяет имя формата конфигурации или ссылки по сигнатуре.

        Args:
            content: Текстовое содержимое конфигурации.

        Returns:
            str: Имя формата ('v2ray', 'clash', 'sing-box', 'xray', 'happ', 'incy', 'throne' или 'unknown').
        """
        text = content.strip()
        if not text:
            return "unknown"

        # 1. Ссылки по протоколам URI
        first_line = text.splitlines()[0].strip().lower()
        if first_line.startswith(("vmess://", "vless://", "trojan://", "ss://", "shadowsocks://")):
            return "v2ray"
        elif first_line.startswith(("happ://", "hiddify://")):
            return "happ"
        elif first_line.startswith("incy://"):
            return "incy"
        elif first_line.startswith("throne://"):
            return "throne"

        # 2. Пробуем парсить как JSON
        if text.startswith("{") and text.endswith("}"):
            try:
                data = json.loads(text)
                if isinstance(data, dict):
                    if "outbounds" in data:
                        outbounds = data.get("outbounds", [])
                        if isinstance(outbounds, list) and outbounds:
                            first_outbound = outbounds[0]
                            if isinstance(first_outbound, dict):
                                if "type" in first_outbound:
                                    return "sing-box"
                                elif "protocol" in first_outbound:
                                    # Отличие Xray от V2Ray: presence of reality/vision/mux/xray fields
                                    out_str = json.dumps(outbounds).lower()
                                    if "reality" in out_str or "vision" in out_str or "mux" in out_str:
                                        return "xray"
                                    return "v2ray"
                        return "sing-box"
                    elif "nodes" in data:
                        return "incy"
                    elif "servers" in data:
                        return "throne"
                    elif "profiles" in data:
                        return "happ"
            except json.JSONDecodeError:
                pass

        # 3. Пробуем парсить как YAML Clash
        if "proxies:" in text or "proxy-groups:" in text or "rules:" in text:
            try:
                yaml_data = yaml.safe_load(text)
                if isinstance(yaml_data, dict) and ("proxies" in yaml_data or "proxy-groups" in yaml_data):
                    return "clash"
            except yaml.YAMLError:
                pass

        return "unknown"


@log_call
def detect_config_format(content: str) -> str:
    """Утилитарная функция автоопределения формата."""
    return FormatDetector.detect(content)


@log_call
def parse_config(content: str) -> ParseResult:
    """
    Универсальная функция парсинга: определяет формат и возвращает ParseResult.

    Args:
        content: Конфигурационный текст или URL.

    Returns:
        ParseResult: Спарсенный результат.
    """
    format_name = FormatDetector.detect(content)
    parser_cls = FormatDetector.PARSERS.get(format_name)

    if not parser_cls:
        result = ParseResult(format_name=format_name)
        result.errors.append(f"Неизвестный или неподдерживаемый формат конфигурации: '{format_name}'.")
        return result

    parser = parser_cls()
    return parser.parse(content)
