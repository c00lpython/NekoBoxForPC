# NekoBoxPlusForPC — Project ROADMAP

## Milestone Tracker

| Status | Name | Description | Tech Details / Agent Notes |
| :---: | :--- | :--- | :--- |
| `[x]` | **Phase 1: PC Modules Scan & Core Spec** | Scan & discover all PC modules (`sing-box-plus`, `adblock`, `amneziawg-go`, `ByeDPI`, `MasterDNSVPN`) | Core protocols, module scanner, strict typing |
| `[x]` | **Phase 1: Core Engine Architecture** | Normalized configs, validators, TUN controller, logging decorator | Python + C-ABI / CLI bindings, 100% tests |
| `[ ]` | **Phase 2: Backend Management & API** | Profile parsers, subscriptions, routing generator, ByeDPI chains | Dynamic config compiler, plugin hooks |
| `[ ]` | **Phase 2: Scripting & Plugins** | Balancers, auto-failover, custom rules extensions | Isolated Python scripting runtime |
| `[ ]` | **Phase 2: CLI Console MVP** | Headless runner for Windows 7+, Linux, macOS | CLI commands, stats monitor, test runs |
| `[ ]` | **Phase 3: Flutter UI Skeleton** | Carcass from sketches (`Panel`, `Configs`, `Groups`, `Routes`, etc.) | Material You, MVVM architecture |
| `[ ]` | **Phase 3: UI-Backend Integration** | Connect Flutter UI to Backend Core via IPC / FFI | Real-time traffic, logs, latency ping |
| `[ ]` | **Phase 3: Animations & Polish** | Micro-interactions, dark theme, smooth charts | High fidelity desktop UX |
| `[ ]` | **Phase 3: Multiplatform Production Builds** | Windows 10+ Installer/Portable, Linux AppImage/deb, macOS dmg | Automated CI/CD release pipeline |

---

### Phase 1 Breakdown: PC Modules & Core Foundation
- `[x]` Specification & Architecture Contract (`docs/SPEC.md`).
- `[ ]` Universal Logger with `@log_call` breadcrumb tracing (`core/utils/logger.py`).
- `[ ]` PC Module Scanner & Capabilities Registry (`core/modules/registry.py`).
- `[ ]` ByeDPI Engine & CLI Arguments Validator (`core/modules/byedpi.py`).
- `[ ]` AdBlock & uBlock Ruleset Engine (`core/modules/adblock.py`).
- `[ ]` MasterDNSVPN Bridge & Fallback Resolver (`core/modules/masterdnsvpn.py`).
- `[ ]` AmneziaWG Obfuscation Engine (`core/modules/amneziawg.py`).
- `[ ]` Core Config Models & Normalizer (`core/config/models.py`, `core/config/normalizer.py`).
- `[ ]` Cross-Platform TUN & System Proxy Controller (`core/network/tun.py`).
- `[ ]` Core Process Lifecycle Manager (`core/engine/manager.py`).
- `[ ]` Complete Multi-Tier Test Suite (`tests/unit/`, `tests/integration/`, `tests/functional/`, `tests/extreme/`, `tests/cyber_safety/`).
