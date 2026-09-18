"""
Юнит-тесты для парсеров конфигураций, логирования и конвертера sing-box.
"""

import base64
import json
import logging
import pytest

from core.config.models import AppConfig, ProfileConfig, ProtocolType, TUNStack
from core.parser.clash import ClashParser
from core.parser.converter import to_singbox_json
from core.parser.detector import FormatDetector, detect_config_format, parse_config
from core.parser.happ import HappParser
from core.parser.incy import IncyParser
from core.parser.singbox import SingBoxParser
from core.parser.throne import ThroneParser
from core.parser.v2ray import V2RayParser
from core.parser.xray import XrayParser
from utils.logger import get_logger, log_call


def test_logger_decorator():
    """Тестирование логирующего декоратора @log_call и хлебных крошек."""
    logger = get_logger("test_module")
    logger.setLevel(logging.DEBUG)

    @log_call
    def inner_func():
        return 42

    @log_call
    def outer_func():
        return inner_func()

    result = outer_func()
    assert result == 42


def test_logger_exception_reraise():
    """Тестирование повторного проброса исключений в @log_call."""
    @log_call
    def failing_func():
        raise ValueError("Test error")

    with pytest.raises(ValueError, match="Test error"):
        failing_func()


def test_v2ray_parser_vless_link():
    """Тестирование парсинга VLESS ссылки."""
    uri = "vless://12345678-1234-1234-1234-123456789abc@example.com:443?type=tcp&security=tls&sni=example.com#VLESS%20Server"
    parser = V2RayParser()
    res = parser.parse(uri)

    assert res.format_name == "v2ray"
    assert len(res.profiles) == 1
    profile = res.profiles[0]
    assert profile.name == "VLESS Server"
    assert profile.protocol == ProtocolType.VLESS
    assert profile.server == "example.com"
    assert profile.server_port == 443
    assert profile.settings["uuid"] == "12345678-1234-1234-1234-123456789abc"
    assert profile.settings["sni"] == "example.com"


def test_v2ray_parser_vmess_link():
    """Тестирование парсинга VMess ссылки."""
    vmess_dict = {
        "v": "2",
        "ps": "VMess Test",
        "add": "vmess.example.com",
        "port": "8443",
        "id": "87654321-4321-4321-4321-cba987654321",
        "aid": "0",
        "scy": "auto",
        "net": "ws",
        "type": "none",
        "host": "vmess.example.com",
        "path": "/ws",
        "tls": "tls",
    }
    b64_str = base64.b64encode(json.dumps(vmess_dict).encode()).decode()
    uri = f"vmess://{b64_str}"

    parser = V2RayParser()
    res = parser.parse(uri)

    assert len(res.profiles) == 1
    profile = res.profiles[0]
    assert profile.name == "VMess Test"
    assert profile.protocol == ProtocolType.VMESS
    assert profile.server == "vmess.example.com"
    assert profile.server_port == 8443
    assert profile.settings["uuid"] == "87654321-4321-4321-4321-cba987654321"


def test_v2ray_parser_trojan_and_ss():
    """Тестирование парсинга Trojan и Shadowsocks ссылок."""
    trojan_uri = "trojan://secretpass@trojan.com:443#TrojanNode"
    ss_uri = "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ=@ss.com:8388#SSNode"

    parser = V2RayParser()
    res1 = parser.parse(trojan_uri)
    assert len(res1.profiles) == 1
    assert res1.profiles[0].protocol == ProtocolType.TROJAN
    assert res1.profiles[0].settings["password"] == "secretpass"

    res2 = parser.parse(ss_uri)
    assert len(res2.profiles) == 1
    assert res2.profiles[0].protocol == ProtocolType.SHADOWSOCKS
    assert res2.profiles[0].settings["password"] == "password"


def test_clash_parser():
    """Тестирование парсера Clash YAML."""
    yaml_content = """
proxies:
  - name: "Clash VLESS"
    type: vless
    server: clash.example.com
    port: 443
    uuid: 12345678-1234-1234-1234-123456789abc
    tls: true
    servername: clash.example.com
  - name: "Clash Hysteria2"
    type: hysteria2
    server: hy2.example.com
    port: 8443
    auth: secret
"""
    parser = ClashParser()
    res = parser.parse(yaml_content)

    assert res.format_name == "clash"
    assert len(res.profiles) == 2
    assert res.profiles[0].name == "Clash VLESS"
    assert res.profiles[0].protocol == ProtocolType.VLESS
    assert res.profiles[1].name == "Clash Hysteria2"
    assert res.profiles[1].protocol == ProtocolType.HYSTERIA2


def test_singbox_parser():
    """Тестирование парсера родногo sing-box JSON."""
    singbox_json = json.dumps({
        "outbounds": [
            {
                "type": "vless",
                "tag": "singbox-vless",
                "server": "sb.example.com",
                "server_port": 443,
                "uuid": "12345678-1234-1234-1234-123456789abc"
            },
            {
                "type": "direct",
                "tag": "direct"
            }
        ]
    })
    parser = SingBoxParser()
    res = parser.parse(singbox_json)

    assert res.format_name == "sing-box"
    assert len(res.profiles) == 1
    assert res.profiles[0].tag == "singbox-vless"
    assert res.profiles[0].server == "sb.example.com"


def test_xray_parser():
    """Тестирование парсера Xray JSON (REALITY / Vision)."""
    xray_json = json.dumps({
        "outbounds": [
            {
                "protocol": "vless",
                "tag": "xray-reality",
                "settings": {
                    "vnext": [{
                        "address": "xray.example.com",
                        "port": 443,
                        "users": [{"id": "uuid-xray", "flow": "xtls-rprx-vision"}]
                    }]
                },
                "streamSettings": {
                    "network": "tcp",
                    "security": "reality",
                    "realitySettings": {
                        "publicKey": "pbk123"
                    }
                }
            }
        ]
    })
    parser = XrayParser()
    res = parser.parse(xray_json)

    assert res.format_name == "xray"
    assert len(res.profiles) == 1
    assert res.profiles[0].server == "xray.example.com"


def test_happ_parser():
    """Тестирование парсера Happ."""
    happ_uri = "happ://happ.example.com:443?type=vless&sni=happ.example.com#HappNode"
    parser = HappParser()
    res = parser.parse(happ_uri)

    assert res.format_name == "happ"
    assert len(res.profiles) == 1
    assert res.profiles[0].name == "HappNode"


def test_incy_parser():
    """Тестирование парсера Incy."""
    incy_uri = "incy://incy.example.com:443?proto=vless#IncyNode"
    parser = IncyParser()
    res = parser.parse(incy_uri)

    assert res.format_name == "incy"
    assert len(res.profiles) == 1
    assert res.profiles[0].name == "IncyNode"


def test_throne_parser():
    """Тестирование парсера Throne."""
    throne_uri = "throne://throne.example.com:443?mode=vless#ThroneNode"
    parser = ThroneParser()
    res = parser.parse(throne_uri)

    assert res.format_name == "throne"
    assert len(res.profiles) == 1
    assert res.profiles[0].name == "ThroneNode"


def test_format_detector_all_formats():
    """Тестирование автоопределения всех 7 форматов."""
    v2ray_link = "vless://uuid@host:443?sni=host#name"
    clash_yaml = "proxies:\n  - name: test\n    type: ss"
    singbox_js = '{"outbounds": [{"type": "vless", "tag": "sb"}]}'
    xray_js = '{"outbounds": [{"protocol": "vless", "tag": "xray", "streamSettings": {"security": "reality"}}]}'
    happ_link = "happ://host:443#happ"
    incy_link = "incy://host:443#incy"
    throne_link = "throne://host:443#throne"

    assert detect_config_format(v2ray_link) == "v2ray"
    assert detect_config_format(clash_yaml) == "clash"
    assert detect_config_format(singbox_js) == "sing-box"
    assert detect_config_format(xray_js) == "xray"
    assert detect_config_format(happ_link) == "happ"
    assert detect_config_format(incy_link) == "incy"
    assert detect_config_format(throne_link) == "throne"


def test_parse_config_universal():
    """Тестирование универсальной точки входа parse_config."""
    link = "vless://12345678-1234-1234-1234-123456789abc@universal.com:443#UniversalTest"
    res = parse_config(link)

    assert res.format_name == "v2ray"
    assert len(res.profiles) == 1
    assert res.profiles[0].server == "universal.com"


def test_converter_to_singbox_json():
    """Тестирование генерации итогового JSON для sing-box."""
    profile = ProfileConfig(
        name="Main Proxy",
        protocol=ProtocolType.VLESS,
        server="proxy.example.com",
        server_port=443,
        settings={"uuid": "my-uuid", "security": "tls", "sni": "proxy.example.com"},
    )
    app_config = AppConfig()
    app_config.tun.enabled = True
    app_config.tun.stack = TUNStack.MIXED

    config_dict = to_singbox_json([profile], app_config)

    assert "log" in config_dict
    assert "dns" in config_dict
    assert "inbounds" in config_dict
    assert "outbounds" in config_dict
    assert "route" in config_dict

    outbound = config_dict["outbounds"][0]
    assert outbound["type"] == "vless"
    assert outbound["server"] == "proxy.example.com"
    assert outbound["uuid"] == "my-uuid"
