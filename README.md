# NekoBox for Android

[![API](https://img.shields.io/badge/API-23%2B-brightgreen.svg?style=flat)](https://android-arsenal.com/api?level=23)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-orange.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Disclaimer

> This project is intended solely for technical research and code learning purposes and does not provide any form of network proxy service. Please do not use this project for any activities that violate local laws and regulations. Do not use this project in production environments. Users are fully responsible for any risks that may arise from using this project. If you download or reference this project, please delete all related content within 24 hours and avoid long-term storage, distribution, or dissemination of any part of this project. **The author reserves the right to modify, update, or remove any part of this project or its contents at any time without prior notice.**

## Downloads

[4pda](https://4pda.to/forum/index.php?showtopic=1121122)

## Системные требования

| Платформа | Архитектура | Статус |
|-----------|-------------|--------|
| Windows 10/11 | x86_64 (64-bit) | ✅ Полная поддержка |
| Windows 8.1 | x86_64 (64-bit) | ✅ Поддерживается |
| Windows 7 SP1 | x86_64 (64-bit) | ⚠️ Доступна портабельная сборка (Schannel/static CRT) |
| Windows 7 SP1 | x86 (32-bit) | ⚠️ Доступна портабельная сборка (Schannel/static CRT) |
| Linux | x86_64 (64-bit) | ✅ Полная поддержка |
| macOS | arm64 (Apple Silicon) | ✅ Полная поддержка |

### Windows 7 (x64 и x86)

> **Портабельные сборки.** Rust CLI (`nbpfpc`) для Windows 7 собирается с `--features win7-tls` (Schannel вместо rustls) и статическим CRT (`+crt-static`). Доступны готовые пакеты для **Windows 7 x64** и **Windows 7 x86 (32-bit)** в каталоге `builds/`. Вся конфигурация, логи и кэш сохраняются автономно в локальной папке `data/` рядом с исполняемым файлом. При запуске на Windows 7 автоматически используется legacy-бинарник `singbox.exe` (если доступен).

## NekoBoxPlusForPC CLI (`nbpfpc`)

В проект включена десктопная консольная утилита и супервайзер ядра `sing-box` на Rust (`nbpfpc`):
- Поддержка синтаксиса команд с дефисом (`-help`, `-ping`, `-list`, `-backup`, `-recovery`) и без (`help`, `ping`, `list`).
- Изолированный запуск без TUN (Mixed SOCKS5/HTTP `127.0.0.1:20808` + Web UI Clash API `127.0.0.1:9090`).
- Параллельный замер задержек (`-ping [TARGET] [OPTIONS]`): режимы вывода (`--mode table|list|pager`), сортировка (`--sort fastest|slowest|name|group|original`), сохранение пинга в LocalStorage и выгрузка отчетов (`--export report.json`).
- Встроенные ассеты (`assets install` / `list`): готовые группы `WARP` (AWG + MASQUE HTTP/3) и `Goida Group` (Free Sub).
- Управление правилами маршрутизации (`newrule`, `delrule`, `ruleslist`, `moverule`, `switchrule`).
- Локальное хранилище баз GeoIP и GeoSite (`geolist`, `geoupdate`, `geofetch`, `geodel`).
- Резервное копирование и восстановление (`-backup` и `-recovery`): интерактивное окно подтверждения, фильтры `CONFIGS`, `ROUTES`, `SETTINGS`, режимы `--mode merge` (безопасное слияние) и `hard`.
- Интерактивный терминальный TUI-пейджер (`list`) и меню настроек (`settings`).

Подробное руководство по установке и командам: [docs/CLI_USAGE.md](docs/CLI_USAGE.md).

## Supported Protocols

* SOCKS (4/4a/5)
* HTTP(S)
* SSH
* Shadowsocks
* ShadowsocksR
* VMess
* Trojan
* VLESS
* AnyTLS/AnyReality
* Snell 1/2/3/4/5
* ShadowTLS
* TUIC
* Juicity
* Hysteria 1/2
* WireGuard
* AmneziaWG 2.0
* Trojan-Go (trojan-go-plugin)
* NaïveProxy
* Mieru (mieru-plugin)
* ByeDPI
* MasterDnsVPN

<details>
<summary>XHTTP Extra TLS Configuration Example</summary>

<pre><code class="language-json">
{
	"headers": {
		"User-Agent": "Mozilla/5.0"
	},
	"no_grpc_header": false,
	"x_padding_bytes": "100-10000",
	"sc_max_each_post_bytes": 1000000,
	"sc_min_posts_interval_ms": 30,
	"xmux": {
		"max_concurrency": "16-32",
		"max_connections": "0-0",
		"c_max_reuse_times": "0-0",
		"h_max_request_times": "600-900",
		"h_max_reusable_secs": "1800-3000",
		"h_keep_alive_period": 0
	},
    "x_padding_obfs_mode": false,
    "x_padding_key": "",
    "x_padding_header": "",
    "x_padding_placement": "",
    "x_padding_method": "",
    "uplink_http_method": "",
    "session_placement": "",
    "session_key": "",
    "session_id_table": "",  // hex/base36/base62/alphabet etc, ""=UUID
    "session_id_length": 0,
    "seq_placement": "",
    "seq_key": "",
    "uplink_data_placement": "",
    "uplink_data_key": "",
    "uplink_chunk_size": 0,
    "congestion_controller": "",  // H3 only: ""=bbr, bbr/cubic/reno
    "cwnd": 0,  // H3 only, default 32
	"download": {
		"mode": "auto",
		"host": "b.yourdomain.com",
		"path": "/xhttp",
		"headers": {
			"User-Agent": "Mozilla/5.0"
		},
		"no_grpc_header": false,
		"x_padding_bytes": "100-10000",
		"sc_max_each_post_bytes": 1000000,
		"sc_min_posts_interval_ms": 30,
		"xmux": {
			"max_concurrency": "16-32",
			"max_connections": "0-0",
			"c_max_reuse_times": "0-0",
			"h_max_request_times": "600-900",
			"h_max_reusable_secs": "1800-3000",
			"h_keep_alive_period": 0
		},
        "x_padding_obfs_mode": false,
        "x_padding_key": "",
        "x_padding_header": "",
        "x_padding_placement": "",
        "x_padding_method": "",
        "uplink_http_method": "",
        "session_placement": "",
        "session_key": "",
        "session_id_table": "",  // hex/base36/base62/alphabet etc, ""=UUID
        "session_id_length": 0,
        "seq_placement": "",
        "seq_key": "",
        "uplink_data_placement": "",
        "uplink_data_key": "",
        "uplink_chunk_size": 0,
        "congestion_controller": "",  // H3 only: ""=bbr, bbr/cubic/reno
        "cwnd": 0,  // H3 only, default 32
		"server": "$(ip_or_domain_of_your_cdn)",
		"server_port": 443,
		"tls": {
			"enabled": true,
			"server_name": "b.yourdomain.com",
			"alpn": "h2",
			"utls": {
				"enabled": true,
				"fingerprint": "chrome"
			}
		}
	}
}
</code></pre>
</details>

<details>
<summary>XHTTP Extra Reality Configuration Example</summary>

<pre><code class="language-json">
{
	"headers": {
		"User-Agent": "Mozilla/5.0"
	},
	"no_grpc_header": false,
	"x_padding_bytes": "100-10000",
	"sc_max_each_post_bytes": 1000000,
	"sc_min_posts_interval_ms": 30,
	"xmux": {
		"max_concurrency": "16-32",
		"max_connections": "0-0",
		"c_max_reuse_times": "0-0",
		"h_max_request_times": "600-900",
		"h_max_reusable_secs": "1800-3000",
		"h_keep_alive_period": 0
	},
    "x_padding_obfs_mode": false,
    "x_padding_key": "",
    "x_padding_header": "",
    "x_padding_placement": "",
    "x_padding_method": "",
    "uplink_http_method": "",
    "session_placement": "",
    "session_key": "",
    "session_id_table": "",  // hex/base36/base62/alphabet etc, ""=UUID
    "session_id_length": 0,
    "seq_placement": "",
    "seq_key": "",
    "uplink_data_placement": "",
    "uplink_data_key": "",
    "uplink_chunk_size": 0,
    "congestion_controller": "",  // H3 only: ""=bbr, bbr/cubic/reno
    "cwnd": 0,  // H3 only, default 32
	"download": {
		"mode": "auto",
		"host": "example.com",
		"path": "/xhttp",
		"headers": {
			"User-Agent": "Mozilla/5.0"
		},
		"no_grpc_header": false,
		"x_padding_bytes": "100-10000",
		"sc_max_each_post_bytes": 1000000,
		"sc_min_posts_interval_ms": 30,
		"xmux": {
			"max_concurrency": "16-32",
			"max_connections": "0-0",
			"c_max_reuse_times": "0-0",
			"h_max_request_times": "600-900",
			"h_max_reusable_secs": "1800-3000",
			"h_keep_alive_period": 0
		},
        "x_padding_obfs_mode": false,
        "x_padding_key": "",
        "x_padding_header": "",
        "x_padding_placement": "",
        "x_padding_method": "",
        "uplink_http_method": "",
        "session_placement": "",
        "session_key": "",
        "session_id_table": "",  // 可选: hex/base36/base62/alphabet 等, 留空则用 UUID
        "session_id_length": 0,
        "seq_placement": "",
        "seq_key": "",
        "uplink_data_placement": "",
        "uplink_data_key": "",
        "uplink_chunk_size": 0,
        "congestion_controller": "",  // H3 only: ""=bbr, bbr/cubic/reno
        "cwnd": 0,  // H3 only, default 32
		"server": "$(ip_or_domain_of_your_cdn)",
		"server_port": 443,
		"tls": {
			"enabled": true,
			"server_name": "example.com",
			"reality": {
				"enabled": true,
				"public_key": "$(your_publicKey)",
				"short_id": "$(your_shortId)"
			},
			"utls": {
				"enabled": true,
				"fingerprint": "chrome"
			}
		}
	}
}
</code></pre>
</details>

## Supported Subscription Format

* Some widely used formats (like Shadowsocks, ClashMeta and v2rayN)
* Remnawave (only with Happ / v2RayTun spoof)
* sing-box outbound

Only resolving outbound, i.e. nodes, is supported. Information such as diversion rules are ignored.

## Building

This project is built from several sibling repositories. The expected layout is:

```text
workspace/
  NekoBoxForAndroid/
  sing-box/
  libneko/
  net/
  amneziawg-go/
  MasterDnsVPN-plus/
  byedpi/
```

To get this structure you should clone original repositories first:

```bash
git clone https://github.com/starifly/NekoBoxForAndroid.git NekoBoxForAndroid
git clone --branch 1.12.x https://github.com/starifly/sing-box.git sing-box
git clone https://github.com/masterking32/MasterDnsVPN.git MasterDnsVPN-plus
```

The patch bundle for those repositories is distributed as files you've applied to those repositories after cloning them (assuming that you did since you're reading this, patch files have everything you need).

After applying the main patches, both `NekoBoxForAndroid/patches` and `sing-box/patches` contain additional patches needed by local dependencies.

Patch `amneziawg-go` manually:

```bash
git clone https://github.com/amnezia-vpn/amneziawg-go.git amneziawg-go
git -C amneziawg-go checkout {replace with patch target commit from the patch file itself}
git -C amneziawg-go apply ../sing-box/patches/amneziawg-go/*.patch
```

Patch `golang.org/x/net` manually:

```bash
git clone https://go.googlesource.com/net net
git -C net checkout {replace with patch target commit from the patch file itself}
git -C net apply ../sing-box/patches/x-slash-net/*.patch
```

`byedpi`, `sing-box`, and `libneko` are handled by `buildScript/lib/core/get_source.sh` when they are missing; `byedpi` is patched automatically from `NekoBoxForAndroid/patches/byedpi`. Edit `buildScript/lib/core/get_source_env.sh` to point the app to the correct repositories and commits: it's important to have correct 
commit hashes there or you'll end up with broken build.

Install Android SDK/NDK, Docker or a Docker-compatible Podman setup, and Android Studio's JBR. Then download runtime assets and build the Go core inside the container:

```bash
cd NekoBoxForAndroid
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export JAVA_HOME=/opt/android-studio/jbr

bash buildScript/lib/assets.sh
bash buildScript/lib/core.docker.sh
```

The containerized core build is required because it builds with the patched Go runtime used by this project. The runtime retains Go's standard monotonic timer behavior; Android device/network wake callbacks restore WG/AWG binds and keepalives without periodic background wakeups. Rebuild the image after changing Go versions, the Dockerfile, or a Go runtime patch:

```bash
bash buildScript/lib/core.docker.sh --rebuild-image
```

Useful configuration knobs for `core.docker.sh`:

- `SING_BOX_SRC`, `X_NET_SRC`, `LIBNEKO_SRC`, `AMNEZIAWG_GO_SRC`, `MASTERDNSVPN_SRC`, `BYEDPI_SRC`: override sibling source locations.
- `AAR_OUT_DIR`, `GO_CACHE_DIR`, `GO_MOD_CACHE_DIR`: override core output and Go cache directories.
- `GO_VERSION`, `BOOTSTRAP_GO_VERSION`, `GO_PATCH_DIR`, `DOCKERFILE`, `PATCHED_GO_ANDROID_IMAGE`: customize the patched Go build image. Runtime patches are applied from `GO_PATCH_DIR` in lexical order.
- `DOCKER_RUN_EXTRA_ARGS`: append Docker or Podman runtime flags, for example `--network host`.

Finally, build the Android app:

```bash
JAVA_HOME=/opt/android-studio/jbr ./gradlew assembleOssDebug
```

For release variants, use the matching Gradle task, for example `assembleFdroidRelease`, and provide signing values through `local.properties` or the `KEYSTORE_PASS`, `ALIAS_NAME`, and `ALIAS_PASS` environment variables.

## Credits

Core:

- [SagerNet/sing-box](https://github.com/SagerNet/sing-box)
- [starifly/sing-box](https://github.com/starifly/sing-box)

Android GUI:

- [shadowsocks/shadowsocks-android](https://github.com/shadowsocks/shadowsocks-android)
- [SagerNet/SagerNet](https://github.com/SagerNet/SagerNet)
- [MatsuriDayo/NekoBoxForAndroid](https://github.com/MatsuriDayo/NekoBoxForAndroid)

Web Dashboard:

- [metacubexd](https://github.com/MetaCubeX/metacubexd)
