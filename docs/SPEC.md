# NekoBoxPlusForPC (NB4PC) — Technical Specification (SPEC.md)

## 1. Overview & Vision
`NekoBoxPlusForPC` is a modern, high-performance, cross-platform Desktop VPN / Proxy client built on top of `sing-box-plus` with native enhancements:
- **Core Engine:** `sing-box-plus` with `libcore` extensions.
- **Enhanced Modules:** `adblock-rust`, `adblock-resources`, `uBlock`, `amneziawg-go`, `ByeDPI`, `MasterDNSVPN`.
- **Target OS:** Windows 10/11 (GUI + CLI), Windows 7 (CLI Core), Linux (x86_64, aarch64), macOS (Apple Silicon / Intel).
- **Architecture Layers:**
  1. `Core Engine Layer` (Go / C-ABI / Native binaries) — High throughput, TUN/TAP, protocol multiplexing.
  2. `Backend Manager & Plugins Layer` (Python + Dart Manager) — Config validation, dynamic routing rules, ByeDPI tuning, plugin hooks, subscriptions auto-updater.
  3. `UI Presentation Layer` (Flutter Desktop) — Modern responsive Material You design, dark theme, statistics panels, interactive logs.

---

## 2. PC Modules & Capabilities Contract

| Module | Purpose | Interface / Integration Contract |
| :--- | :--- | :--- |
| `sing-box-plus` | Primary proxy engine | CLI daemon / IPC JSON configuration / Clash & V2Ray APIs |
| `adblock-rust` / `uBlock` | DNS & URL rule blocking | Fast matching engine, EasyList ruleset loader, regex filtering |
| `amneziawg-go` | DPI-resistant WireGuard | Custom headers `Jc, Jmin, Jmax, S1, S2, H1..H4` obfuscation |
| `ByeDPI` | DPI circumvention proxy | CLI options generator (`--split`, `--disoob`, `--auto`, `--fake`, `--ttl`, `--sni`), local SOCKS5/HTTP inbound |
| `MasterDNSVPN` | Multi-upstream secure DNS | Encrypted DNS resolver (DoH, DoT, DoQ), latency tester, fallback failover |
| `Wintun / TUN` | System-wide VPN adapter | Native virtual network adapter configuration & route tables |

---

## 3. Configuration & Schema Contracts

### 3.1 Profile / Outbound Schema
All profiles are normalized into a unified DTO model:
- `id`: UUID string
- `name`: Human readable label
- `protocol`: `shadowsocks`, `vmess`, `vless`, `trojan`, `hysteria2`, `wireguard`, `amneziawg`, `tuic`, `ssh`, `direct`, `block`
- `server`: IP or Domain
- `server_port`: 1..65535
- `settings`: Protocol-specific typed dict (keys, UUID, passwords, transport, TLS/uTLS, ECH, Amnezia headers)
- `multiplex`: Optional mux config (smux, yamux, h2mux)
- `byedpi_config`: Optional linked ByeDPI upstream bypass chain

### 3.2 Routing & RuleSet Schema
- `rules`: Ordered list of routing decisions (GeoIP, GeoSite, Domain, IP CIDR, Port, Process name)
- `outbound`: Tag of target outbound (`direct`, `proxy`, `block`, `bypass-byedpi`, `warp`)

---

## 4. Cross-Platform Network Controller
- **Windows:** Controls `Wintun.dll`, configures default gateway and DNS via `netsh` / Win32 IP Helper API.
- **Linux:** Manages `/dev/net/tun`, `ip route`, `iptables` / `nftables`.
- **macOS:** Manages `utun` devices, `route` and `networksetup` DNS.
- **System Proxy Fallback:** Native Windows Registry `Internet Settings`, GNOME/KDE proxy settings, macOS `networksetup -setwebproxy`.
