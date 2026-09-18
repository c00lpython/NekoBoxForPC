"""
Модуль логирования и трейсинга вызовов NekoBoxPlusForPC.
Предоставляет декоратор @log_call для ведения breadcrumb-цепочек вызовов
и расчёта длительности выполнения функций.
"""

import functools
import logging
import threading
import time
import traceback
from typing import Any, Callable

# Thread-local хранилище стека хлебных крошек (breadcrumb chain)
_thread_local = threading.local()


def _get_call_stack() -> list[str]:
    """
    Получает текущий стек хлебных крошек для текущего потока.

    Returns:
        list[str]: Список имён декорированных функций в стеке.
    """
    if not hasattr(_thread_local, "call_stack"):
        _thread_local.call_stack = []
    return _thread_local.call_stack


def get_logger(name: str) -> logging.Logger:
    """
    Получает настроенный логгер для модуля.

    Args:
        name: Имя модуля (обычно __name__).

    Returns:
        logging.Logger: Экземпляр логгера.
    """
    logger = logging.getLogger(f"nb4pc.{name}")
    if not logger.handlers:
        handler = logging.StreamHandler()
        formatter = logging.Formatter(
            "[%(asctime)s] [%(levelname)s] [%(name)s] %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S",
        )
        handler.setFormatter(formatter)
        logger.addHandler(handler)
        logger.setLevel(logging.WARNING)  # По умолчанию молчит
    return logger


def log_call(func: Callable[..., Any]) -> Callable[..., Any]:
    """
    Декоратор для логирования входа/выхода и ошибок функции.

    1. На DEBUG логирует вход функции с breadcrumb-цепочкой.
    2. На DEBUG логирует успешный выход и длительность выполнения.
    3. На ERROR логирует исключение с полным трейсбеком и повторно выбрасывает его (re-raise).

    Args:
        func: Декорируемая функция.

    Returns:
        Callable: Обернутая функция.
    """
    logger = get_logger(func.__module__)

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        func_qualname = f"{func.__module__}.{func.__qualname__}"
        stack = _get_call_stack()
        stack.append(func_qualname)
        breadcrumb = " > ".join(stack)

        start_time = time.perf_counter()
        if logger.isEnabledFor(logging.DEBUG):
            logger.debug(f"ENTER: [{breadcrumb}]")

        try:
            result = func(*args, **kwargs)
            duration_ms = (time.perf_counter() - start_time) * 1000
            if logger.isEnabledFor(logging.DEBUG):
                logger.debug(
                    f"EXIT: [{breadcrumb}] (Duration: {duration_ms:.2f}ms)"
                )
            return result
        except Exception as exc:
            duration_ms = (time.perf_counter() - start_time) * 1000
            tb_str = traceback.format_exc()
            logger.error(
                f"FAIL: [{breadcrumb}] after {duration_ms:.2f}ms - {exc}\n{tb_str}"
            )
            raise
        finally:
            if stack and stack[-1] == func_qualname:
                stack.pop()

    return wrapper
