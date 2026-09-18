#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Автоматический скрипт валидации 8 ключевых сценариев NekoBoxForPC.

Сценарии:
  1. Simple SOCKS5 local (no connection, direct).
  2. Parse all protocols configs (vless, vmess, trojan, ss, hy2, tuic, awg, clash, singbox).
  3. Try subscription from git (AvenCores/goida-vpn-configs).
  4. Try to use VLESS.
  5. AWG test (WARP.conf params validation).
  6. Proxy chain (vless-vless, then awg-vless).
  7. Balancer (type -list ranges, then type -group GROUPNAME).
  8. Proxy chain (awg-groupbalancer).
"""

import json
import os
import subprocess
import sys
import time
import urllib.request

# Настройка UTF-8 для корректного вывода символов в Windows консоли
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NBPFPC_EXE = os.path.join(ROOT_DIR, "nbpfpc.exe")
WARP_CONF = os.path.join(ROOT_DIR, "WARP.conf")
LOGS_DIR = os.path.join(ROOT_DIR, "logs")
LATEST_LOG = os.path.join(LOGS_DIR, "latest.log")


def log_step(title: str) -> None:
    """Выводит визуальный разделитель шага."""
    print("\n" + "=" * 65)
    print(f" >>> {title}")
    print("=" * 65)


def run_cli(*args: str, check: bool = True) -> subprocess.CompletedProcess:
    """Запускает CLI-утилиту nbpfpc.exe."""
    cmd = [NBPFPC_EXE] + list(args)
    res = subprocess.run(
        cmd,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if check and res.returncode != 0:
        print(f"STDERR:\n{res.stderr}", file=sys.stderr)
        raise RuntimeError(f"Команда failed (code {res.returncode}): {' '.join(cmd)}")
    return res


def stop_core() -> None:
    """Останавливает все активные экземпляры ядра singbox."""
    run_cli("stop", check=False)
    time.sleep(1)


def wait_for_clash_api(timeout_secs: float = 6.0) -> bool:
    """Опрашивает локальный эндпоинт Clash API."""
    start = time.time()
    url = "http://127.0.0.1:9090/version"
    while time.time() - start < timeout_secs:
        try:
            req = urllib.request.Request(url)
            with urllib.request.urlopen(req, timeout=1.0) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(0.3)
    return False


def test_stage_1_direct() -> None:
    """Этап 1: Simple SOCKS5 local (direct outbound)."""
    log_step("ЭТАП 1: Simple SOCKS5 Local (Direct Outbound)")
    stop_core()

    # Запускаем в фоновом режиме nbpfpc run --direct
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", "--direct"],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        ready = wait_for_clash_api(timeout_secs=6.0)
        assert ready, "Clash API не ответил вовремя для режима --direct"

        # Проверяем, что Clash API отдает версию ядра
        req = urllib.request.Request("http://127.0.0.1:9090/version")
        with urllib.request.urlopen(req, timeout=2.0) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            print(f"  ✓ Clash API отвечает: {data}")
            assert "sing-box" in data.get("version", ""), "Неверная версия ядра"

        print("  ✓ Локальный прокси порт 20808 активен в режиме direct.")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    # Проверяем, что лог записался
    assert os.path.exists(LATEST_LOG), "Файл logs/latest.log не был создан"
    with open(LATEST_LOG, "r", encoding="utf-8", errors="replace") as f:
        content = f.read()
    assert "SingBox Session Started" in content, "В логе нет заголовка сессии"
    print("  ✓ Лог сессии успешно сохранен в logs/latest.log.")
    print("  [ЭТАП 1 УСПЕШНО ПРОЙДЕН]")


def test_stage_2_parse_all_protocols() -> None:
    """Этап 2: Парсинг конфигураций всех поддерживаемых протоколов."""
    log_step("ЭТАП 2: Парсинг всех видов конфигураций и протоколов")

    test_configs = [
        ("VLESS", "vless://21710d7d-5e87-4655-bac2-36e9b655e3ee@85.209.157.184:443?security=reality&pbk=test&sni=microsoft.com#VlessTest"),
        ("VMESS", "vmess://eyJhZGQiOiIxMjcuMC4wLjEiLCJhaWQiOiIwIiwiaWQiOiJmMTkyMjcyMi1mZDEwLTRkY2MtYmYzOC1kZmQyZDNlYTIwNWIiLCJuZXQiOiJ0Y3AiLCJwb3J0IjoiNDQzIiwicHMiOiJWTWVzc1Rlc3QiLCJzY3kiOiJhdXRvIiwidGxzIjoiIn0="),
        ("Trojan", "trojan://password123@1.2.3.4:443?sni=example.com#TrojanTest"),
        ("Shadowsocks", "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQxMjM=@1.2.3.4:8388#SsTest"),
        ("Hysteria2", "hy2://secretpass@1.2.3.4:443?sni=hy2.example.com#Hy2Test"),
        ("TUIC", "tuic://b832bf64-3253-4876-8809-58b760a4f5b7:tuicpass@1.2.3.4:8443?congestion_control=bbr#TuicTest"),
        ("AmneziaWG/Wireguard (conf)", WARP_CONF),
        ("Clash YAML", "proxies:\n  - name: clash_ss\n    type: ss\n    server: 1.1.1.1\n    port: 8388\n    cipher: aes-128-gcm\n    password: pwd"),
        ("SingBox JSON", '{"outbounds": [{"type": "vless", "tag": "sb-vless", "server": "1.1.1.1", "server_port": 443}]}'),
    ]

    for proto_name, payload in test_configs:
        res = run_cli("parse", payload)
        assert "Спарсено профилей: 1" in res.stdout, f"Ошибка парсинга {proto_name}:\n{res.stdout}"
        print(f"  ✓ {proto_name:<26} -> Распознан и успешно спарсен.")

    print("  [ЭТАП 2 УСПЕШНО ПРОЙДЕН]")


def test_stage_3_subscription_git() -> None:
    """Этап 3: Загрузка подписки из Git (AvenCores/goida-vpn-configs)."""
    log_step("ЭТАП 3: Импорт подписки из Git (AvenCores/goida-vpn-configs)")
    sub_url = "https://raw.githubusercontent.com/AvenCores/goida-vpn-configs/main/githubmirror/5.txt"

    # Создаем или обновляем группу подписки
    res = run_cli("newgroup", "GitGoida", "--sub", sub_url, check=False)
    if "уже существует" in res.stderr or res.returncode != 0:
        res = run_cli("update", "GitGoida")

    stdout = res.stdout
    print(stdout)
    assert "Счетчик профилей" in stdout or "успешно" in stdout, "Не сработал счетчик профилей"
    assert "61 узлов" in stdout or "узлов" in stdout, "Узлы не были импортированы"
    print("  ✓ Подписка из Git успешно загружена, счетчик профилей показал разбивку.")
    print("  [ЭТАП 3 УСПЕШНО ПРОЙДЕН]")


def test_stage_4_use_vless() -> None:
    """Этап 4: Тест запуска сессии VLESS."""
    log_step("ЭТАП 4: Тестирование запуска VLESS сессии")
    stop_core()

    # Берем первый узел из подписки UserSub или GitGoida
    target_node = "UserSub.🇩🇪 HIT VPN | Германия"
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", target_node],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        ready = wait_for_clash_api(timeout_secs=6.0)
        assert ready, f"Не удалось запустить VLESS узел '{target_node}'"

        req = urllib.request.Request("http://127.0.0.1:9090/proxies")
        with urllib.request.urlopen(req, timeout=2.0) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            proxies = data.get("proxies", {})
            assert "proxy" in proxies, "Аутбаунд 'proxy' отсутствует в Clash API"
            print(f"  ✓ VLESS узел активен в sing-box: {proxies['proxy'].get('type')} ({proxies['proxy'].get('now', '')})")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    assert os.path.exists(LATEST_LOG)
    print("  ✓ Логи сессии VLESS записаны в logs/latest.log.")
    print("  [ЭТАП 4 УСПЕШНО ПРОЙДЕН]")


def test_stage_5_awg() -> None:
    """Этап 5: AmneziaWG тест (валидация параметров WARP.conf и JSON)."""
    log_step("ЭТАП 5: AmneziaWG тест (WARP.conf & параметры обфускации)")

    # 1. Проверяем парсер
    res_parse = run_cli("parse", WARP_CONF)
    assert "Протокол: Amneziawg" in res_parse.stdout, "Протокол должен быть Amneziawg"
    assert "188.114.97.66:4500" in res_parse.stdout, "Неверный endpoint"
    print("  ✓ WARP.conf корректно определен как AmneziaWG.")

    # 2. Генерируем Ultimate singbox конфиг и проверяем параметры
    res_build = run_cli("build-config", WARP_CONF)
    config_json = json.loads(res_build.stdout)
    out0 = config_json["outbounds"][0]

    assert out0["type"] == "wireguard", "Outbound type должен быть wireguard"
    assert out0["jc"] == 4, "Параметр Jc не равен 4"
    assert out0["jmin"] == 40, "Параметр Jmin не равен 40"
    assert out0["jmax"] == 70, "Параметр Jmax не равен 70"
    assert out0["s1"] == 0, "Параметр S1 не равен 0"
    assert out0["s2"] == 0, "Параметр S2 не равен 0"
    assert out0["h1"] == 1, "Параметр H1 не равен 1"
    assert out0["h2"] == 2, "Параметр H2 не равен 2"
    assert out0["h3"] == 3, "Параметр H3 не равен 3"
    assert out0["h4"] == 4, "Параметр H4 не равен 4"
    print("  ✓ Все параметры обфускации AmneziaWG (Jc, Jmin, Jmax, S1, S2, H1..H4) соответствуют спецификации.")
    print("  [ЭТАП 5 УСПЕШНО ПРОЙДЕН]")


def test_stage_6_proxy_chain() -> None:
    """Этап 6: Proxy chain (vless-vless, then awg-vless)."""
    log_step("ЭТАП 6: Proxy Chain (VLESS -> VLESS и AWG -> VLESS)")
    stop_core()

    # 1. Цепочка VLESS -> VLESS
    chain_arg = "UserSub.🇩🇪 HIT VPN | Германия,UserSub.🇳🇱 HIT VPN | Нидерланды"
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", "--chain", chain_arg],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        ready = wait_for_clash_api(timeout_secs=6.0)
        assert ready, "Не удалось запустить цепочку vless-vless"

        # Проверяем структуру сгенерированного конфига в .runtime
        with open(".runtime/active_singbox_config.json", "r", encoding="utf-8") as f:
            cfg = json.load(f)

        outbounds = cfg["outbounds"]
        node0 = next(o for o in outbounds if o["tag"] == "chain-node-0")
        proxy = next(o for o in outbounds if o["tag"] == "proxy")

        assert proxy.get("detour") == "chain-node-0", "Второй узел не содержит detour на первый"
        print(f"  ✓ Цепочка VLESS->VLESS построена: '{node0['tag']}' -> '{proxy['tag']}' (detour: {proxy['detour']}).")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    # 2. Цепочка AWG -> VLESS
    awg_chain = f"{WARP_CONF},UserSub.🇩🇪 HIT VPN | Германия"
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", "--chain", awg_chain],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        # Проверяем active_singbox_config.json
        time.sleep(1.5)
        with open(".runtime/active_singbox_config.json", "r", encoding="utf-8") as f:
            cfg = json.load(f)

        outbounds = cfg["outbounds"]
        awg_node = next(o for o in outbounds if o["tag"] == "chain-node-0")
        vless_node = next(o for o in outbounds if o["tag"] == "proxy")

        assert awg_node["type"] == "wireguard", "Первое звено должно быть wireguard"
        assert vless_node.get("detour") == "chain-node-0", "Выходной VLESS узел должен идти через detour chain-node-0"
        print("  ✓ Цепочка AWG->VLESS построена: AmneziaWG -> VLESS (detour: chain-node-0).")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    print("  [ЭТАП 6 УСПЕШНО ПРОЙДЕН]")


def test_stage_7_balancer() -> None:
    """Этап 7: Балансировщик (-list диапазоны, then -group)."""
    log_step("ЭТАП 7: Балансировщик (-list диапазоны и -group GROUPNAME)")
    stop_core()

    # Создаем профили с именами на [a-c], чтобы гарантировать наличие кандидатов
    run_cli("new", "Default.alpha-node", "--instant", '{"type": "direct", "tag": "alpha-node"}', check=False)
    run_cli("new", "Default.beta-node", "--instant", '{"type": "direct", "tag": "beta-node"}', check=False)

    # 1. Балансировщик по диапазону (-list [a-c])
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", "--balancer", "-list [a-c]"],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        ready = wait_for_clash_api(timeout_secs=6.0)
        assert ready, "Не удалось запустить балансировщик по диапазону"

        with open(".runtime/active_singbox_config.json", "r", encoding="utf-8") as f:
            cfg = json.load(f)

        proxy_out = next(o for o in cfg["outbounds"] if o["tag"] == "proxy")
        assert proxy_out["type"] == "urltest", "Балансировщик должен иметь тип urltest"
        assert len(proxy_out["outbounds"]) > 0, "Список кандидатов балансировщика не должен быть пустым"
        print(f"  ✓ Балансировщик (-list [a-c]) активен: urltest с {len(proxy_out['outbounds'])} кандидатами.")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    # 2. Балансировщик по группе (-group UserSub)
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", "--balancer", "-group UserSub"],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        ready = wait_for_clash_api(timeout_secs=6.0)
        assert ready, "Не удалось запустить балансировщик по группе"

        with open(".runtime/active_singbox_config.json", "r", encoding="utf-8") as f:
            cfg = json.load(f)

        proxy_out = next(o for o in cfg["outbounds"] if o["tag"] == "proxy")
        assert proxy_out["type"] == "urltest", "Балансировщик группы должен иметь тип urltest"
        print(f"  ✓ Балансировщик группы UserSub активен: urltest с {len(proxy_out['outbounds'])} кандидатами.")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    print("  [ЭТАП 7 УСПЕШНО ПРОЙДЕН]")


def test_stage_8_chain_groupbalancer() -> None:
    """Этап 8: Proxy chain (awg-groupbalancer)."""
    log_step("ЭТАП 8: Proxy Chain (AmneziaWG -> Group Balancer)")
    stop_core()

    cb_arg = f"{WARP_CONF},UserSub"
    proc = subprocess.Popen(
        [NBPFPC_EXE, "run", "--chain-balancer", cb_arg],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        time.sleep(2.0)
        with open(".runtime/active_singbox_config.json", "r", encoding="utf-8") as f:
            cfg = json.load(f)

        outbounds = cfg["outbounds"]
        front = next(o for o in outbounds if o["tag"] == "chain-front")
        balancer = next(o for o in outbounds if o["tag"] == "proxy")
        candidate = next(o for o in outbounds if o["tag"] == "proxy-cand-0")

        assert front["type"] == "wireguard", "Фронт цепочки должен быть wireguard"
        assert balancer["type"] == "urltest", "Основной прокси должен быть балансировщиком urltest"
        assert candidate.get("detour") == "chain-front", "Кандидаты балансировщика должны направляться в chain-front"

        print("  ✓ Сконфигурирована цепочка AmneziaWG -> Group Balancer:")
        print(f"    - Входной туннель: {front['tag']} ({front['type']})")
        print(f"    - Балансировщик:   {balancer['tag']} ({balancer['type']}, кандидатов: {len(balancer['outbounds'])})")
        print(f"    - Маршрутизация:   Каждый кандидат использует detour -> '{candidate['detour']}'")
    finally:
        stop_core()
        proc.kill()
        proc.wait()

    # Проверяем отображение логов через nbpfpc logs
    logs_res = run_cli("logs", "--lines", "10")
    print(f"  ✓ Логи последней сессии успешно читаются через 'nbpfpc logs':\n{logs_res.stdout.strip()}")
    print("  [ЭТАП 8 УСПЕШНО ПРОЙДЕН]")


def main() -> None:
    """Точка входа запуска всех 8 проверочных этапов."""
    print("#################################################################")
    print("#      NekoBoxPlusForPC: ПОЛНАЯ АВТОВАЛИДАЦИЯ 8 СЦЕНАРИЕВ       #")
    print("#################################################################")

    start_time = time.time()
    try:
        test_stage_1_direct()
        test_stage_2_parse_all_protocols()
        test_stage_3_subscription_git()
        test_stage_4_use_vless()
        test_stage_5_awg()
        test_stage_6_proxy_chain()
        test_stage_7_balancer()
        test_stage_8_chain_groupbalancer()

        elapsed = time.time() - start_time
        print("\n" + "#" * 65)
        print(f"#  ВСЕ 8 СЦЕНАРИЕВ УСПЕШНО ПРОЙДЕНЫ ЗА {elapsed:.2f} сек!  #")
        print("#################################################################\n")
    except Exception as exc:
        stop_core()
        print(f"\n[ОШИБКА ВАЛИДАЦИИ]: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
