# Документация и разбор модуля libcore

Полный подробный разбор структуры, назначения и всех файлов модуля `libcore` находится в файле [`libcore/TRACE.md`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/libcore/TRACE.md).

## Краткая сводка:
- **`libcore`** — это высокопроизводительное прокси-ядро на языке Go, служащее мостом между Java/Android/PC слоем NekoBox и прокси-библиотекой `sing-box`.
- Включает кастомные протоколы обхода блокировок: **ByeDPI**, **TrustTunnel**, **Juicity**, **MasterDNS VPN**.
- Реализует платформенные интерфейсы Android TUN/VpnService (`platform_tun.go`, `protect.go`, `procfs/`).
- Содержит AdBlock-фильтр (`adblock.go` на базе Rust `adblock-rust` и Cronet), диагностику задержек (URLTest, ICMP Ping), измерение скорости (Speedtest) и определение типов NAT (STUN).

## Кроссплатформенная сборка бинарников (`singbox/`):
Скрипты сборки `build_desktop.cmd` и `build_desktop.ps1` создают готовые бинарные файлы в следующей структуре:
- `singbox/Windows/singbox.exe` — Windows 10 / 11 (x86_64)
- `singbox/Windows7/singbox.exe` — Windows 7 / 8 / 8.1 (x86_64 legacy)
- `singbox/Linux/singbox` — Linux (x86_64)
- `singbox/MacOS/singbox` — macOS (arm64 Apple Silicon)

Каждый бинарник включает полные версии 6 подмодулей (`adblock-rust`, `adblock-resources`, `uBlock`, `amneziawg-go`, `byedpi`, `MasterDnsVPN`) и набор тегов сборки `with_conntrack,with_gvisor,with_quic,with_dhcp,with_wireguard,with_awg,with_tailscale,with_openvpn,with_openconnect,with_utls,with_acme,with_clash_api,with_ccm,with_ocm,with_grpc,badlinkname,tfogo_checklinkname0`.
