"""
Базовые сущности и интерфейсы для парсеров конфигураций.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, List, Optional

from core.config.models import ProfileConfig
from utils.logger import log_call


@dataclass
class ParseResult:
    """Результат разбора конфигурации парсером."""

    profiles: List[ProfileConfig] = field(default_factory=list)
    format_name: str = "unknown"
    raw_config: Any = None
    errors: List[str] = field(default_factory=list)

    @property
    def is_success(self) -> bool:
        """Проверяет, успешны ли результаты парсинга (есть ли хотя бы 1 валидный профиль)."""
        return len(self.profiles) > 0 and len(self.errors) == 0


class BaseParser(ABC):
    """Абстрактный базовый класс для всех парсеров конфигураций."""

    format_name: str = "base"

    @abstractmethod
    @log_call
    def parse(self, content: str) -> ParseResult:
        """
        Разбирает текстовое содержимое конфигурации или ссылку.

        Args:
            content: Строковое представление (URI ссылку, JSON, YAML).

        Returns:
            ParseResult: Результат парсинга с имеющимися профилями и ошибками.
        """
        pass
