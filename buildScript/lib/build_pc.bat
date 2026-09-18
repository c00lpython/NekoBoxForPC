@echo off
echo Building NekoBoxForPC CLI client...
cd %~dp0..\..\libcore
go build -ldflags="-checklinkname=0" -tags="with_gvisor,with_quic,with_wireguard,with_awg,with_tailscale,with_utls,with_clash_api,badlinkname,tfogo_checklinkname0" -o nekobox-cli.exe ./cmd/cli
if %ERRORLEVEL% EQU 0 (
    echo Build successful: libcore\nekobox-cli.exe
) else (
    echo Build failed.
    exit /b %ERRORLEVEL%
)
