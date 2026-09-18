"""
Пакет универсального парсера конфигураций для NekoBoxPlusForPC.
Поддерживает форматы: v2ray, happ, incy, throne, clash, sing-box, xray.
"""

from core.parser.base import BaseParser, ParseResult
from core.parser.detector import FormatDetector, detect_config_format, parse_config

__all__ = [
    "BaseParser",
    "ParseResult",
    "FormatDetector",
    "detect_config_format",
    "parse_config",
]
