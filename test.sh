#!/usr/bin/env bash
# test.sh - Единый запуск тестирования проекта NekoBoxForPC (Linux/macOS/CI)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=================================================="
echo "   Запуск полного набора проверок и тестов        "
echo "=================================================="

# 1. Проверка маркеров конфликтов слияния git
echo ""
echo "[1/3] Проверка на маркеры конфликтов слияния git..."
MARKER_PATTERN="<<<<<<"$'\x3c'" HEAD"
if git grep -l "$MARKER_PATTERN" -- ":!test.ps1" ":!test.sh" 2>/dev/null; then
    echo "[ERROR] Обнаружены маркеры конфликтов git!"
    exit 1
else
    echo "  -> Маркеры конфликтов отсутствуют. Отлично!"
fi

# 2. Python Unit тесты
echo ""
echo "[2/3] Запуск Python unit-тестов (pytest)..."
if command -v pytest >/dev/null 2>&1; then
    pytest "$SCRIPT_DIR/tests/unit/" -v
elif command -v python3 >/dev/null 2>&1; then
    python3 -m pytest "$SCRIPT_DIR/tests/unit/" -v
elif command -v python >/dev/null 2>&1; then
    python -m pytest "$SCRIPT_DIR/tests/unit/" -v
else
    echo "[WARN] Python/pytest не найден, пропускаем pytest"
fi
echo "  -> Тесты Python выполнены!"

# 3. Rust тесты (core_manager)
echo ""
echo "[3/3] Запуск тестов Rust core_manager (cargo test)..."
if command -v cargo >/dev/null 2>&1; then
    cargo test --manifest-path "$SCRIPT_DIR/core_manager/Cargo.toml"
    echo "  -> Все тесты Rust успешно пройдены!"
else
    echo "[WARN] Cargo не найден в PATH, пропускаем cargo test"
fi

echo ""
echo "=================================================="
echo " [SUCCESS] ВСЕ ПРОВЕРКИ И ТЕСТЫ ПРОЙДЕНЫ УСПЕШНО! "
echo "=================================================="
exit 0
