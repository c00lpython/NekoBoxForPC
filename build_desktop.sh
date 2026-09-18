#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

resolve_version() {
	local dir="$1" mode="${2:-}" version=""
	[ -d "$dir" ] || { printf '<not set>'; return 0; }
	if [ "$mode" = "tag" ]; then
		version="$(git -C "$dir" describe --tags --abbrev=0 2>/dev/null || true)"
	else
		version="$(git -C "$dir" describe --tags --exact-match 2>/dev/null || true)"
		if [ -z "$version" ]; then
			version="$(git -C "$dir" rev-parse --short HEAD 2>/dev/null || true)"
		fi
	fi
	printf '%s' "${version:-<not set>}"
}

VER_AMNEZIA="$(resolve_version "$SCRIPT_DIR/../amneziawg-go" tag)"
VER_BYEDPI="$(resolve_version "$SCRIPT_DIR/../byedpi")"
VER_MASTERDNS="$(resolve_version "$SCRIPT_DIR/../MasterDnsVPN-plus" tag)"
VER_ADBLOCK_RUST="$(resolve_version "$SCRIPT_DIR/../adblock-rust")"
VER_ADBLOCK_RES="$(resolve_version "$SCRIPT_DIR/../adblock-resources")"
VER_UBLOCK="$(resolve_version "$SCRIPT_DIR/../uBlock")"

LDFLAGS="-s -w \
-X libcore.VersionAmnezia=${VER_AMNEZIA} \
-X libcore.VersionByeDPI=${VER_BYEDPI} \
-X libcore.VersionMasterDnsVPN=${VER_MASTERDNS} \
-X libcore.VersionAdblockRust=${VER_ADBLOCK_RUST} \
-X libcore.VersionAdblockResources=${VER_ADBLOCK_RES} \
-X libcore.VersionUBlock=${VER_UBLOCK}"

TAGS="with_conntrack,with_gvisor,with_quic,with_wireguard,with_awg,with_utls,with_clash_api"

echo "=================================================="
echo " Building sing-box (libcore edition) binaries "
echo " Modules resolved:"
echo "   amneziawg-go      : ${VER_AMNEZIA}"
echo "   byedpi            : ${VER_BYEDPI}"
echo "   MasterDnsVPN-plus : ${VER_MASTERDNS}"
echo "=================================================="

cd "$SCRIPT_DIR/libcore"

echo "[1/3] Compiling for Windows (x86_64)..."
GOOS=windows GOARCH=amd64 CGO_ENABLED=0 go build -v -ldflags "$LDFLAGS" -tags "$TAGS" -o "$SCRIPT_DIR/singbox/Windows/singbox.exe" ./cmd/cli
echo " -> Output: singbox/Windows/singbox.exe"

echo "[2/3] Compiling for Linux (x86_64)..."
GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -v -ldflags "$LDFLAGS" -tags "$TAGS" -o "$SCRIPT_DIR/singbox/Linux/singbox" ./cmd/cli
echo " -> Output: singbox/Linux/singbox"

echo "[3/3] Compiling for macOS (arm64 Apple Silicon)..."
GOOS=darwin GOARCH=arm64 CGO_ENABLED=0 go build -v -ldflags "$LDFLAGS" -tags "$TAGS" -o "$SCRIPT_DIR/singbox/MacOS/singbox" ./cmd/cli
echo " -> Output: singbox/MacOS/singbox"

echo "=================================================="
echo " Build completed successfully! "
echo " Binaries located in ./singbox/ [Windows|Linux|MacOS]"
echo "=================================================="
