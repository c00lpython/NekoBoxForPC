# test.ps1 - Единый запуск тестирования проекта NekoBoxForPC
param(
    [switch]$Fast
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "   Запуск полного набора проверок и тестов        " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 1. Проверка маркеров конфликтов слияния git
Write-Host ""
Write-Host "[1/3] Проверка на маркеры конфликтов слияния git..." -ForegroundColor Yellow
$markerPattern = "<<<<<<" + "< HEAD"
$conflictMarkers = git grep -l "$markerPattern" -- ":!test.ps1" ":!test.sh" 2>$null

if ($LASTEXITCODE -eq 0 -and $conflictMarkers) {
    Write-Host "[ERROR] Обнаружены маркеры конфликтов git:" -ForegroundColor Red
    foreach ($line in $conflictMarkers) {
        Write-Host "  - $line" -ForegroundColor Red
    }
    exit 1
} else {
    Write-Host "  -> Маркеры конфликтов отсутствуют. Отлично!" -ForegroundColor Green
}

# 2. Python Unit тесты
Write-Host ""
Write-Host "[2/3] Запуск Python unit-тестов (pytest)..." -ForegroundColor Yellow
python -m pytest "$ScriptDir/tests/unit/" -v
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Тесты pytest завершились с ошибкой!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "  -> Все тесты Python успешно пройдены!" -ForegroundColor Green

# 3. Rust тесты (core_manager)
Write-Host ""
Write-Host "[3/3] Запуск тестов Rust core_manager (cargo test)..." -ForegroundColor Yellow
$env:CARGO_TARGET_DIR = "C:\nb_target"
cargo test --manifest-path "$ScriptDir/core_manager/Cargo.toml"
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Тесты Cargo завершились с ошибкой!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "  -> Все тесты Rust успешно пройдены!" -ForegroundColor Green

Write-Host ""
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host " [SUCCESS] ВСЕ ПРОВЕРКИ И ТЕСТЫ ПРОЙДЕНЫ УСПЕШНО! " -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Cyan
exit 0
