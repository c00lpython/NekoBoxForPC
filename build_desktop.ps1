# Build script for sing-box (libcore edition) for Windows, Linux, and macOS
param(
    [string]$TargetOS = "all",
    [switch]$SkipCore,
    [switch]$SkipRust,
    [switch]$SkipPack
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
Write-Host " Building NekoBoxPlusForPC Full Release           " -ForegroundColor Cyan
Write-Host " Modules resolved:" -ForegroundColor Gray
Write-Host "   amneziawg-go      : $VerAmnezia" -ForegroundColor Gray
Write-Host "   byedpi            : $VerByeDPI" -ForegroundColor Gray
Write-Host "   MasterDnsVPN-plus : $VerMasterDNS" -ForegroundColor Gray
Write-Host "   adblock-rust      : $VerAdblockRust" -ForegroundColor Gray
Write-Host "   adblock-resources : $VerAdblockRes" -ForegroundColor Gray
Write-Host "   uBlock            : $VerUBlock" -ForegroundColor Gray
Write-Host "==================================================" -ForegroundColor Cyan

# ============================================================
# STEP 1: Build Go core (sing-box)
# ============================================================
if (-not $SkipCore) {
    Set-Location "$ScriptDir\libcore"

    if ($TargetOS -eq "windows" -or $TargetOS -eq "all") {
        Write-Host "[Core 1/4] Compiling sing-box for Windows (x86_64)..." -ForegroundColor Yellow
        $env:GOOS = "windows"; $env:GOARCH = "amd64"; $env:CGO_ENABLED = "0"
        go build -v "$LdFlagsArg" -tags "$Tags" -o "$ScriptDir\singbox\Windows\singbox.exe" .\cmd\cli
        Write-Host " -> Output: singbox\Windows\singbox.exe" -ForegroundColor Green
    }

    if ($TargetOS -eq "linux" -or $TargetOS -eq "all") {
        Write-Host "[Core 2/4] Compiling sing-box for Linux (x86_64)..." -ForegroundColor Yellow
        $env:GOOS = "linux"; $env:GOARCH = "amd64"; $env:CGO_ENABLED = "0"
        go build -v "$LdFlagsArg" -tags "$Tags" -o "$ScriptDir\singbox\Linux\singbox" .\cmd\cli
        Write-Host " -> Output: singbox\Linux\singbox" -ForegroundColor Green
    }

    if ($TargetOS -eq "macos" -or $TargetOS -eq "all") {
        Write-Host "[Core 3/4] Compiling sing-box for macOS (arm64)..." -ForegroundColor Yellow
        $env:GOOS = "darwin"; $env:GOARCH = "arm64"; $env:CGO_ENABLED = "0"
        go build -v "$LdFlagsArg" -tags "$Tags" -o "$ScriptDir\singbox\MacOS\singbox" .\cmd\cli
        Write-Host " -> Output: singbox\MacOS\singbox" -ForegroundColor Green
    }

    if ($TargetOS -eq "windows7" -or $TargetOS -eq "all") {
        Write-Host "[Core 4/4] Compiling sing-box for Windows 7 (x86_64 legacy)..." -ForegroundColor Yellow
        $env:GOOS = "windows"; $env:GOARCH = "amd64"; $env:CGO_ENABLED = "0"
        go build -v "$LdFlagsArg" -tags "$Tags" -o "$ScriptDir\singbox\Windows7\singbox.exe" .\cmd\cli
        Write-Host " -> Output: singbox\Windows7\singbox.exe" -ForegroundColor Green
    }

    $env:GOOS = ""; $env:GOARCH = ""; $env:CGO_ENABLED = ""
    Set-Location $ScriptDir
} else {
    Write-Host "[SKIP] Go core build skipped (--SkipCore)" -ForegroundColor DarkGray
}

# ============================================================
# STEP 2: Build Rust CLI (nbpfpc)
# ============================================================
if (-not $SkipRust) {
    Set-Location "$ScriptDir\core_manager"

    if ($TargetOS -eq "windows" -or $TargetOS -eq "all") {
        Write-Host "[Rust 1/2] Compiling nbpfpc for Windows (default TLS)..." -ForegroundColor Yellow
        cargo build --release --bin nbpfpc
        Write-Host " -> Output: core_manager\target\release\nbpfpc.exe" -ForegroundColor Green
    }

    if ($TargetOS -eq "windows7" -or $TargetOS -eq "windows7-x64" -or $TargetOS -eq "all") {
        Write-Host "[Rust 2/3] Compiling nbpfpc for Windows 7 x64 (native-tls / Schannel)..." -ForegroundColor Yellow
        $env:RUSTFLAGS = "-C target-feature=+crt-static"
        cargo build --release --bin nbpfpc --no-default-features --features win7-tls
        $env:RUSTFLAGS = ""
        $win7out = "$ScriptDir\release_dist\Windows7"
        New-Item -ItemType Directory -Force -Path $win7out | Out-Null
        Copy-Item "$ScriptDir\core_manager\target\release\nbpfpc.exe" "$win7out\nbpfpc.exe" -Force
        Write-Host " -> Output: release_dist\Windows7\nbpfpc.exe" -ForegroundColor Green
    }

    if ($TargetOS -eq "windows7-x86" -or $TargetOS -eq "windows7" -or $TargetOS -eq "all") {
        Write-Host "[Rust 3/3] Compiling nbpfpc for Windows 7 x86 (32-bit, native-tls / Schannel)..." -ForegroundColor Yellow
        $env:RUSTFLAGS = "-C target-feature=+crt-static"
        cargo build --release --bin nbpfpc --target i686-pc-windows-msvc --no-default-features --features win7-tls
        $env:RUSTFLAGS = ""
        $win7x86out = "$ScriptDir\release_dist\Windows7-x86"
        New-Item -ItemType Directory -Force -Path $win7x86out | Out-Null
        Copy-Item "$ScriptDir\core_manager\target\i686-pc-windows-msvc\release\nbpfpc.exe" "$win7x86out\nbpfpc.exe" -Force
        Write-Host " -> Output: release_dist\Windows7-x86\nbpfpc.exe" -ForegroundColor Green
    }

    if ($TargetOS -eq "linux" -or $TargetOS -eq "all") {
        Write-Host "[Rust 4/5] Compiling nbpfpc for Linux x86_64 (musl)..." -ForegroundColor Yellow
        $env:CARGO_TARGET_DIR = "C:\nb_target"
        cargo zigbuild --release --bin nbpfpc --target x86_64-unknown-linux-musl
        $linuxOut = "$ScriptDir\release_dist\Linux"
        New-Item -ItemType Directory -Force -Path $linuxOut | Out-Null
        Copy-Item "C:\nb_target\x86_64-unknown-linux-musl\release\nbpfpc" "$linuxOut\nbpfpc" -Force
        Write-Host " -> Output: release_dist\Linux\nbpfpc" -ForegroundColor Green
    }

    if ($TargetOS -eq "macos" -or $TargetOS -eq "all") {
        Write-Host "[Rust 5/5] Compiling nbpfpc for macOS (arm64 Apple Silicon)..." -ForegroundColor Yellow
        $env:CARGO_TARGET_DIR = "C:\nb_target"
        cargo zigbuild --release --bin nbpfpc --target aarch64-apple-darwin
        $macOut = "$ScriptDir\release_dist\MacOS"
        New-Item -ItemType Directory -Force -Path $macOut | Out-Null
        Copy-Item "C:\nb_target\aarch64-apple-darwin\release\nbpfpc" "$macOut\nbpfpc" -Force
        Write-Host " -> Output: release_dist\MacOS\nbpfpc" -ForegroundColor Green
    }

    Set-Location $ScriptDir
} else {
    Write-Host "[SKIP] Rust CLI build skipped (--SkipRust)" -ForegroundColor DarkGray
}

# ============================================================
# STEP 3: Assemble distribution packages into release_dist & builds
# ============================================================
if (-not $SkipPack) {
    Write-Host "`n[Pack] Assembling distribution packages..." -ForegroundColor Cyan

    $platforms = @()
    if ($TargetOS -eq "windows" -or $TargetOS -eq "all") { $platforms += "Windows" }
    if ($TargetOS -eq "linux" -or $TargetOS -eq "all") { $platforms += "Linux" }
    if ($TargetOS -eq "macos" -or $TargetOS -eq "all") { $platforms += "MacOS" }
    if ($TargetOS -eq "windows7" -or $TargetOS -eq "all") { $platforms += "Windows7" }
    if ($TargetOS -eq "windows7-x86" -or $TargetOS -eq "windows7" -or $TargetOS -eq "all") { $platforms += "Windows7-x86" }

    $buildsDir = "$ScriptDir\builds"
    New-Item -ItemType Directory -Force -Path $buildsDir | Out-Null

    foreach ($platform in $platforms) {
        $distDir = "$ScriptDir\release_dist\$platform"
        if (Test-Path $distDir) { Remove-Item -Recurse -Force $distDir }
        New-Item -ItemType Directory -Force -Path $distDir | Out-Null

        # Копирование sing-box ядра
        $corePlatform = if ($platform -eq "Windows7-x86") { "Windows7" } else { $platform }
        $coreSrc = "$ScriptDir\singbox\$corePlatform"
        if (Test-Path $coreSrc) {
            Get-ChildItem -Path $coreSrc -Exclude ".gitkeep" | ForEach-Object {
                Copy-Item $_.FullName "$distDir\" -Force
            }
            Write-Host "  [$platform] sing-box core copied" -ForegroundColor DarkGreen
        }

        # Копирование nbpfpc CLI
        $isWin = ($platform -like "Windows*")
        $binName = if ($isWin) { "nbpfpc.exe" } else { "nbpfpc" }
        $rustExe = if ($platform -eq "Windows7-x86") {
            "$ScriptDir\release_dist\Windows7-x86\nbpfpc.exe"
        } elseif ($platform -eq "Windows7") {
            "$ScriptDir\release_dist\Windows7\nbpfpc.exe"
        } elseif ($platform -eq "Linux") {
            "$ScriptDir\release_dist\Linux\nbpfpc"
        } elseif ($platform -eq "MacOS") {
            "$ScriptDir\release_dist\MacOS\nbpfpc"
        } else {
            "$ScriptDir\core_manager\target\release\nbpfpc.exe"
        }

        if (-not (Test-Path $rustExe)) {
            if ($platform -eq "Windows7-x86") {
                $rustExe = "C:\nb_target\i686-pc-windows-msvc\release\nbpfpc.exe"
            } elseif ($platform -eq "Linux") {
                $rustExe = "C:\nb_target\x86_64-unknown-linux-musl\release\nbpfpc"
            } elseif ($platform -eq "MacOS") {
                $rustExe = "C:\nb_target\aarch64-apple-darwin\release\nbpfpc"
            }
        }

        $targetCli = "$distDir\$binName"
        if ((Test-Path $rustExe) -and ((Resolve-Path $rustExe).Path -ne (Resolve-Path $targetCli -ErrorAction SilentlyContinue).Path)) {
            Copy-Item $rustExe $targetCli -Force
            Write-Host "  [$platform] $binName copied" -ForegroundColor DarkGreen
        }

        # Копирование metacubexd Web UI
        $metacubexdSrc = "$ScriptDir\metacubexd"
        if (Test-Path $metacubexdSrc) {
            $metacubexdDst = "$distDir\metacubexd"
            if (Test-Path $metacubexdDst) { Remove-Item -Recurse -Force $metacubexdDst }
            Copy-Item -Recurse $metacubexdSrc $metacubexdDst
            Write-Host "  [$platform] metacubexd UI copied" -ForegroundColor DarkGreen
        }

        # Дублирование готового портабельного каталога в builds/
        $platformBuildDir = "$buildsDir\$platform"
        if (Test-Path $platformBuildDir) { Remove-Item -Recurse -Force $platformBuildDir }
        Copy-Item -Recurse $distDir $platformBuildDir
        Write-Host "  [$platform] Portable build ready in: builds\$platform" -ForegroundColor Green

        # Создание ZIP-архива
        $zipName = if ($platform -eq "Windows7-x86") {
            "NekoBoxPlusForPC-Windows7-x86.zip"
        } else {
            "NekoBoxPlusForPC-$platform-x64.zip"
        }
        $zipPath = "$ScriptDir\release_dist\$zipName"
        try {
            if (Test-Path $zipPath) { Remove-Item $zipPath -Force -ErrorAction SilentlyContinue }
            Compress-Archive -Path "$distDir\*" -DestinationPath $zipPath -Force -ErrorAction Stop
            Copy-Item $zipPath "$buildsDir\$zipName" -Force -ErrorAction SilentlyContinue
            Write-Host "  [$platform] Archive created: release_dist\$zipName & builds\$zipName" -ForegroundColor Green
        } catch {
            $err = $_.Exception.Message
            Write-Host "  [WARN][$platform] Zip archive warning: $err" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "[SKIP] Distribution packing skipped (--SkipPack)" -ForegroundColor DarkGray
}

Write-Host "`n==================================================" -ForegroundColor Cyan
Write-Host " Build completed successfully!                    " -ForegroundColor Green
Write-Host " Distribution: .\release_dist\ [Windows|Windows7|Linux|MacOS]" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
