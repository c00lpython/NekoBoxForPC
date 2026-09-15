# Build script for sing-box (libcore edition) for Windows, Linux, and macOS
param(
    [string]$TargetOS = "all"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Get-GitVersion {
    param([string]$RepoPath, [string]$Mode = "auto")
    if (-not (Test-Path $RepoPath)) { return "<not set>" }
    $ver = ""
    if ($Mode -eq "tag") {
        $ver = cmd /c "git -C `"$RepoPath`" describe --tags --abbrev=0 2>nul"
    }
    if (-not $ver) {
        $ver = cmd /c "git -C `"$RepoPath`" describe --tags --exact-match 2>nul"
    }
    if (-not $ver) {
        $ver = cmd /c "git -C `"$RepoPath`" rev-parse --short HEAD 2>nul"
    }
    if ($ver) { return ([string]$ver).Trim() }
    return "<not set>"
}

$VerAmnezia = Get-GitVersion "$ScriptDir\..\amneziawg-go" "tag"
$VerByeDPI = Get-GitVersion "$ScriptDir\..\byedpi" "auto"
$VerMasterDNS = Get-GitVersion "$ScriptDir\..\MasterDnsVPN-plus" "tag"
$VerAdblockRust = Get-GitVersion "$ScriptDir\..\adblock-rust" "auto"
$VerAdblockRes = Get-GitVersion "$ScriptDir\..\adblock-resources" "auto"
$VerUBlock = Get-GitVersion "$ScriptDir\..\uBlock" "auto"

$LdFlagsArg = "-ldflags=-s -w -checklinkname=0 -X libcore.VersionAmnezia=$VerAmnezia -X libcore.VersionByeDPI=$VerByeDPI -X libcore.VersionMasterDnsVPN=$VerMasterDNS -X libcore.VersionAdblockRust=$VerAdblockRust -X libcore.VersionAdblockResources=$VerAdblockRes -X libcore.VersionUBlock=$VerUBlock"
$Tags = "with_conntrack,with_gvisor,with_quic,with_dhcp,with_wireguard,with_awg,with_tailscale,with_openvpn,with_openconnect,with_utls,with_acme,with_clash_api,with_ccm,with_ocm,with_grpc,badlinkname,tfogo_checklinkname0"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host " Building sing-box (libcore edition) binaries " -ForegroundColor Cyan
Write-Host " Modules resolved:" -ForegroundColor Gray
Write-Host "   amneziawg-go      : $VerAmnezia" -ForegroundColor Gray
Write-Host "   byedpi            : $VerByeDPI" -ForegroundColor Gray
Write-Host "   MasterDnsVPN-plus : $VerMasterDNS" -ForegroundColor Gray
Write-Host "   adblock-rust      : $VerAdblockRust" -ForegroundColor Gray
Write-Host "   adblock-resources : $VerAdblockRes" -ForegroundColor Gray
Write-Host "   uBlock            : $VerUBlock" -ForegroundColor Gray
Write-Host "==================================================" -ForegroundColor Cyan

Set-Location "$ScriptDir\libcore"

if ($TargetOS -eq "windows" -or $TargetOS -eq "all") {
    Write-Host "[1/3] Compiling for Windows (x86_64)..." -ForegroundColor Yellow
    $env:GOOS = "windows"
    $env:GOARCH = "amd64"
    $env:CGO_ENABLED = "0"
    $out = "$ScriptDir\singbox\Windows\singbox.exe"
    go build -v "$LdFlagsArg" -tags "$Tags" -o "$out" .\cmd\cli
    Write-Host " -> Output: singbox\Windows\singbox.exe" -ForegroundColor Green
}

if ($TargetOS -eq "linux" -or $TargetOS -eq "all") {
    Write-Host "[2/3] Compiling for Linux (x86_64)..." -ForegroundColor Yellow
    $env:GOOS = "linux"
    $env:GOARCH = "amd64"
    $env:CGO_ENABLED = "0"
    $out = "$ScriptDir\singbox\Linux\singbox"
    go build -v "$LdFlagsArg" -tags "$Tags" -o "$ScriptDir\singbox\Linux\singbox" .\cmd\cli
    Write-Host " -> Output: singbox\Linux\singbox" -ForegroundColor Green
}

if ($TargetOS -eq "macos" -or $TargetOS -eq "all") {
    Write-Host "[3/4] Compiling for macOS (arm64 Apple Silicon)..." -ForegroundColor Yellow
    $env:GOOS = "darwin"
    $env:GOARCH = "arm64"
    $env:CGO_ENABLED = "0"
    $out = "$ScriptDir\singbox\MacOS\singbox"
    go build -v "$LdFlagsArg" -tags "$Tags" -o "$ScriptDir\singbox\MacOS\singbox" .\cmd\cli
    Write-Host " -> Output: singbox\MacOS\singbox" -ForegroundColor Green
}

if ($TargetOS -eq "windows7" -or $TargetOS -eq "all") {
    Write-Host "[4/4] Compiling for Windows 7 (x86_64 legacy)..." -ForegroundColor Yellow
    $env:GOOS = "windows"
    $env:GOARCH = "amd64"
    $env:CGO_ENABLED = "0"
    $out = "$ScriptDir\singbox\Windows7\singbox.exe"
    go build -v "$LdFlagsArg" -tags "$Tags" -o "$out" .\cmd\cli
    Write-Host " -> Output: singbox\Windows7\singbox.exe" -ForegroundColor Green
}

# Reset environment variables
$env:GOOS = ""
$env:GOARCH = ""
$env:CGO_ENABLED = ""

Set-Location $ScriptDir

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host " Build completed successfully! " -ForegroundColor Green
Write-Host " Binaries located in .\singbox\ [Windows|Windows7|Linux|MacOS]" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
