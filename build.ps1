# build.ps1 - Унифицированный скрипт сборки NekoBoxForPC
param(
    [Parameter(Position=0)]
    [ValidateSet("core", "cli", "app", "all", "test", "help")]
    [string]$Target = "help",

    [Parameter(Position=1)]
    [string]$TargetOS = "current",

    [switch]$Dist,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Show-Usage {
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host " NekoBoxForPC - Единый скрипт сборки (build.ps1)                " -ForegroundColor Cyan
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host "Использование:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1 <target> [target_os] [-Dist]" -ForegroundColor White
    Write-Host ""
    Write-Host "Доступные цели <target>:" -ForegroundColor Yellow
    Write-Host "  core   - Сборка ядра sing-box (Go/libcore)" -ForegroundColor White
    Write-Host "           Примеры: .\build.ps1 core (для текущей ОС)" -ForegroundColor Gray
    Write-Host "                    .\build.ps1 core all (для всех ОС: Win, Linux, Mac)" -ForegroundColor Gray
    Write-Host "  cli    - Сборка менеджера ядра nbpfpc (Rust core_manager)" -ForegroundColor White
    Write-Host "           Примеры: .\build.ps1 cli (сборка release в корень проекта)" -ForegroundColor Gray
    Write-Host "                    .\build.ps1 cli all -Dist (сборка всех дистрибутивов в release_dist)" -ForegroundColor Gray
    Write-Host "  app    - Десктопное приложение (экспериментальный статус)" -ForegroundColor White
    Write-Host "  all    - Полная сборка доступных компонентов (core + cli)" -ForegroundColor White
    Write-Host "  test   - Запуск полного набора проверок и тестов (test.ps1)" -ForegroundColor White
    Write-Host "  help   - Вызов этой справки" -ForegroundColor White
    Write-Host "=================================================================" -ForegroundColor Cyan
}

if ($Help -or $Target -eq "help") {
    Show-Usage
    exit 0
}

function Build-Core {
    param([string]$OS)
    Write-Host "===> [CORE] Запуск сборки sing-box ядра..." -ForegroundColor Cyan
    $osArg = "windows"
    if ($OS -eq "all") { $osArg = "all" }
    elseif ($OS -eq "linux") { $osArg = "linux" }
    elseif ($OS -eq "macos") { $osArg = "macos" }
    elseif ($OS -eq "windows7") { $osArg = "windows7" }

    if (Test-Path "$ScriptDir\build_desktop.ps1") {
        & "$ScriptDir\build_desktop.ps1" -TargetOS $osArg
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] Ошибка при сборке ядра sing-box!" -ForegroundColor Red
            exit $LASTEXITCODE
        }
    } else {
        Write-Host "[ERROR] Скрипт build_desktop.ps1 не найден!" -ForegroundColor Red
        exit 1
    }
}

function Build-Cli {
    param([string]$OS, [bool]$PackDist)
    Write-Host "===> [CLI] Сборка Rust CLI менеджера nbpfpc..." -ForegroundColor Cyan

    $targetDir = "C:\nb_target"
    $env:CARGO_TARGET_DIR = $targetDir

    if ($OS -eq "all" -or $PackDist) {
        Write-Host "-> Режим мультиплатформенной сборки (release_dist)..." -ForegroundColor Yellow
        $distDir = "$ScriptDir\release_dist"
        if (-not (Test-Path $distDir)) { New-Item -ItemType Directory -Path $distDir | Out-Null }

        cargo build --release --manifest-path "$ScriptDir\core_manager\Cargo.toml"
        Copy-Item "$targetDir\release\nbpfpc.exe" "$ScriptDir\nbpfpc.exe" -Force
        Compress-Archive -Path "$targetDir\release\nbpfpc.exe" -DestinationPath "$distDir\nbpfpc-v1.0.0-beta-windows-x64.zip" -Force

        $env:RUSTFLAGS = "-C target-feature=+crt-static"
        cargo build --release --manifest-path "$ScriptDir\core_manager\Cargo.toml"
        $env:RUSTFLAGS = ""
        Compress-Archive -Path "$targetDir\release\nbpfpc.exe" -DestinationPath "$distDir\nbpfpc-v1.0.0-beta-windows7-x64.zip" -Force
    } else {
        Write-Host "-> Сборка для текущей операционной системы..." -ForegroundColor Yellow
        cargo build --release --manifest-path "$ScriptDir\core_manager\Cargo.toml"
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] Ошибка сборки Cargo!" -ForegroundColor Red
            exit $LASTEXITCODE
        }
        $builtBin = "$targetDir\release\nbpfpc.exe"
        if (-not (Test-Path $builtBin)) {
            $builtBin = "$ScriptDir\core_manager\target\release\nbpfpc.exe"
        }
        if (Test-Path $builtBin) {
            Copy-Item $builtBin "$ScriptDir\nbpfpc.exe" -Force
            Write-Host "-> Исполняемый файл успешно скопирован в корень: .\nbpfpc.exe" -ForegroundColor Green
        }
    }
}

function Build-App {
    Write-Host "===> [APP] Сборка десктопного приложения..." -ForegroundColor Cyan
    Write-Host "[INFO] Компонент 'app' является экспериментальным и находится в стадии активной разработки (GUI Python/PySide6)." -ForegroundColor Yellow
    Write-Host "[INFO] Сборка пакета 'app' в данный момент отключена согласно конфигурации проекта." -ForegroundColor Yellow
}

function Run-Tests {
    Write-Host "===> [TEST] Запуск полного набора тестов..." -ForegroundColor Cyan
    & "$ScriptDir\test.ps1"
}

switch ($Target.ToLower()) {
    "core" { Build-Core $TargetOS }
    "cli"  { Build-Cli $TargetOS $Dist.IsPresent }
    "app"  { Build-App }
    "all"  {
        Build-Core $TargetOS
        Build-Cli $TargetOS $Dist.IsPresent
        Build-App
    }
    "test" { Run-Tests }
}

Write-Host ""
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host " Сборка цели '$Target' завершена! " -ForegroundColor Green
Write-Host "=================================================================" -ForegroundColor Cyan
