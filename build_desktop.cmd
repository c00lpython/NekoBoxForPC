@echo off
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%libcore"

set AMNEZIA_VER=v3.1.20260828
set BYEDPI_VER=ba53229
set MASTERDNS_VER=v2026.06.13.234407-7de2476
set ADBLOCK_RUST_VER=v0.13.2
set ADBLOCK_RES_VER=9a0cc431
set UBLOCK_VER=1.73.0

for /f "tokens=*" %%i in ('git -C "..\amneziawg-go" describe --tags --abbrev=0 2^>nul') do set AMNEZIA_VER=%%i
for /f "tokens=*" %%i in ('git -C "..\byedpi" rev-parse --short HEAD 2^>nul') do set BYEDPI_VER=%%i
for /f "tokens=*" %%i in ('git -C "..\MasterDnsVPN-plus" describe --tags --abbrev=0 2^>nul') do set MASTERDNS_VER=%%i
for /f "tokens=*" %%i in ('git -C "..\adblock-rust" describe --tags --abbrev=0 2^>nul') do set ADBLOCK_RUST_VER=%%i
for /f "tokens=*" %%i in ('git -C "..\adblock-resources" rev-parse --short HEAD 2^>nul') do set ADBLOCK_RES_VER=%%i
for /f "tokens=*" %%i in ('git -C "..\uBlock" describe --tags --abbrev=0 2^>nul') do set UBLOCK_VER=%%i

echo ==================================================
echo  Building sing-box (libcore edition) binaries 
echo  Modules resolved:
echo    amneziawg-go      : %AMNEZIA_VER%
echo    byedpi            : %BYEDPI_VER%
echo    MasterDnsVPN-plus : %MASTERDNS_VER%
echo    adblock-rust      : %ADBLOCK_RUST_VER%
echo    adblock-resources : %ADBLOCK_RES_VER%
echo    uBlock            : %UBLOCK_VER%
echo ==================================================

set LDFLAGS=-s -w -checklinkname=0 -X libcore.VersionAmnezia=%AMNEZIA_VER% -X libcore.VersionByeDPI=%BYEDPI_VER% -X libcore.VersionMasterDnsVPN=%MASTERDNS_VER% -X libcore.VersionAdblockRust=%ADBLOCK_RUST_VER% -X libcore.VersionAdblockResources=%ADBLOCK_RES_VER% -X libcore.VersionUBlock=%UBLOCK_VER%
set TAGS=with_conntrack,with_gvisor,with_quic,with_dhcp,with_wireguard,with_awg,with_tailscale,with_openvpn,with_openconnect,with_utls,with_acme,with_clash_api,with_ccm,with_ocm,with_grpc,badlinkname,tfogo_checklinkname0

echo [1/3] Compiling for Windows (x86_64)...
set GOOS=windows
set GOARCH=amd64
set CGO_ENABLED=0
go build -v -ldflags "%LDFLAGS%" -tags "%TAGS%" -o "..\singbox\Windows\singbox.exe" .\cmd\cli
if errorlevel 1 exit /b %errorlevel%
echo  -^> Output: singbox\Windows\singbox.exe

echo [2/3] Compiling for Linux (x86_64)...
set GOOS=linux
set GOARCH=amd64
set CGO_ENABLED=0
go build -v -ldflags "%LDFLAGS%" -tags "%TAGS%" -o "..\singbox\Linux\singbox" .\cmd\cli
if errorlevel 1 exit /b %errorlevel%
echo  -^> Output: singbox\Linux\singbox

echo [3/4] Compiling for macOS (arm64 Apple Silicon)...
set GOOS=darwin
set GOARCH=arm64
set CGO_ENABLED=0
go build -v -ldflags "%LDFLAGS%" -tags "%TAGS%" -o "..\singbox\MacOS\singbox" .\cmd\cli
if errorlevel 1 exit /b %errorlevel%
echo  -^> Output: singbox\MacOS\singbox

echo [4/4] Compiling for Windows 7 (x86_64 legacy)...
set GOOS=windows
set GOARCH=amd64
set CGO_ENABLED=0
go build -v -ldflags "%LDFLAGS%" -tags "%TAGS%" -o "..\singbox\Windows7\singbox.exe" .\cmd\cli
if errorlevel 1 exit /b %errorlevel%
echo  -^> Output: singbox\Windows7\singbox.exe

echo ==================================================
echo  Build completed successfully!
echo ==================================================
