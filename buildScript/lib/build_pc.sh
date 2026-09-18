#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../../libcore"

echo "Building NekoBoxForPC CLI client..."
go build -ldflags="-checklinkname=0" -tags="with_gvisor,with_quic,with_wireguard,with_awg,with_tailscale,with_utls,with_clash_api,badlinkname,tfogo_checklinkname0" -o nekobox-cli ./cmd/cli

echo "Build successful: libcore/nekobox-cli"
