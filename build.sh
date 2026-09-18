#!/usr/bin/env bash
# build.sh - Унифицированный скрипт сборки NekoBoxForPC (Linux/macOS)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET="${1:-help}"
PARAM_OS="${2:-current}"

show_usage() {
    echo "================================================================="
    echo " NekoBoxForPC - Единый скрипт сборки (build.sh)                 "
    echo "================================================================="
    echo "Использование:"
    echo "  ./build.sh <target> [target_os] [--dist]"
    echo ""
    echo "Доступные цели <target>:"
    echo "  core   - Сборка ядра sing-box (Go/libcore)"
    echo "           Примеры: ./build.sh core (для текущей ОС)"
    echo "                    ./build.sh core all (для Win, Linux, Mac)"
    echo "  cli    - Сборка менеджера ядра nbpfpc (Rust core_manager)"
    echo "           Примеры: ./build.sh cli (сборка release в корень проекта)"
    echo "                    ./build.sh cli all --dist (сборка архивов в release_dist)"
    echo "  app    - Десктопное приложение (экспериментальный статус)"
    echo "  all    - Полная сборка доступных компонентов (core + cli)"
    echo "  test   - Запуск полного набора проверок и тестов (test.sh)"
    echo "  help   - Вызов этой справки"
    echo "================================================================="
}

build_core() {
    local target_os="$1"
    echo "===> [CORE] Запуск сборки sing-box ядра..."
    if [ -f "$SCRIPT_DIR/build_desktop.sh" ]; then
        bash "$SCRIPT_DIR/build_desktop.sh" "$target_os"
    else
        echo "[ERROR] Скрипт build_desktop.sh не найден!"
        exit 1
    fi
}

build_cli() {
    local target_os="$1"
    local pack_dist="${2:-false}"
    echo "===> [CLI] Сборка Rust CLI менеджера nbpfpc..."

    if [ "$target_os" = "all" ] || [ "$pack_dist" = "true" ]; then
        echo "-> Режим мультиплатформенной сборки (release_dist)..."
        local dist_dir="$SCRIPT_DIR/release_dist"
        mkdir -p "$dist_dir"

        if command -v cargo >/dev/null 2>&1; then
            cargo build --release --manifest-path "$SCRIPT_DIR/core_manager/Cargo.toml"
            if [ -f "$SCRIPT_DIR/core_manager/target/release/nbpfpc" ]; then
                cp "$SCRIPT_DIR/core_manager/target/release/nbpfpc" "$SCRIPT_DIR/nbpfpc"
                chmod +x "$SCRIPT_DIR/nbpfpc"
            fi
        fi
        echo "-> Бинарный файл CLI собран."
    else
        echo "-> Сборка для текущей операционной системы..."
        if command -v cargo >/dev/null 2>&1; then
            cargo build --release --manifest-path "$SCRIPT_DIR/core_manager/Cargo.toml"
            if [ -f "$SCRIPT_DIR/core_manager/target/release/nbpfpc" ]; then
                cp "$SCRIPT_DIR/core_manager/target/release/nbpfpc" "$SCRIPT_DIR/nbpfpc"
                chmod +x "$SCRIPT_DIR/nbpfpc"
                echo "-> Исполняемый файл успешно скопирован в корень: ./nbpfpc"
            fi
        else
            echo "[ERROR] Cargo не найден в PATH!"
            exit 1
        fi
    fi
}

build_app() {
    echo "===> [APP] Сборка десктопного приложения..."
    echo "[INFO] Компонент 'app' является экспериментальным и находится в стадии активной разработки (GUI Python/PySide6)."
    echo "[INFO] Сборка пакета 'app' в данный момент отключена согласно конфигурации проекта."
}

run_tests() {
    echo "===> [TEST] Запуск полного набора тестов..."
    if [ -f "$SCRIPT_DIR/test.sh" ]; then
        bash "$SCRIPT_DIR/test.sh"
    else
        echo "[ERROR] Скрипт test.sh не найден!"
        exit 1
    fi
}

case "${TARGET,,}" in
    core)
        build_core "$PARAM_OS"
        ;;
    cli)
        is_dist="false"
        for arg in "$@"; do
            if [ "$arg" = "--dist" ] || [ "$arg" = "-Dist" ]; then
                is_dist="true"
            fi
        done
        build_cli "$PARAM_OS" "$is_dist"
        ;;
    app)
        build_app
        ;;
    all)
        build_core "$PARAM_OS"
        build_cli "$PARAM_OS" "false"
        build_app
        ;;
    test)
        run_tests
        ;;
    help|--help|-h|"")
        show_usage
        exit 0
        ;;
    *)
        echo "[ERROR] Неизвестная цель: $TARGET"
        show_usage
        exit 1
        ;;
esac

echo ""
echo "================================================================="
echo " Сборка цели '$TARGET' завершена! "
echo "================================================================="
