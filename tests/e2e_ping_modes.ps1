# E2E Test for NekoBoxPlusForPC Ping Modes, Sorting, Export, Recovery, and Assets
$ErrorActionPreference = "Stop"

$binPath = ".\nbpfpc.exe"
if (-not (Test-Path $binPath)) {
    Write-Error "Исполняемый файл $binPath не найден! Сначала соберите проект: cargo build"
}

Write-Host "==> [E2E Test] 1. Проверка справки команды ping и recovery..." -ForegroundColor Cyan
$helpOut = & $binPath help ping | Out-String
if ($helpOut -notmatch "--mode" -or $helpOut -notmatch "--sort" -or $helpOut -notmatch "--export") {
    Write-Error "Справка nbpfpc help ping не содержит новых параметров (--mode, --sort, --export)!"
}
$recHelpOut = & $binPath help recovery | Out-String
if ($recHelpOut -notmatch "--mode" -or $recHelpOut -notmatch "--yes") {
    Write-Error "Справка nbpfpc help recovery не содержит параметров --mode и --yes!"
}
Write-Host "    ✓ Справка содержит все необходимые опции." -ForegroundColor Green

Write-Host "==> [E2E Test] 2. Проверка встроенных assets (WARP AWG+MASQUE и Goida Group)..." -ForegroundColor Cyan
$assetsList = & $binPath assets list | Out-String
if ($assetsList -notmatch "WARP" -or $assetsList -notmatch "Goida Group") {
    Write-Error "Команда assets list не показывает группы WARP и Goida Group!"
}
$assetsInst = & $binPath assets install | Out-String
if ($assetsInst -notmatch "WARP" -or $assetsInst -notmatch "Goida Group") {
    Write-Error "Установка assets не завершилась успешно!"
}
Write-Host "    ✓ Ассеты WARP и Goida Group успешно установлены и валидны." -ForegroundColor Green

Write-Host "==> [E2E Test] 3. Проверка режима -mode list на группе WARP..." -ForegroundColor Cyan
$listOut = & $binPath ping WARP -mode list --threads 4 | Out-String
if ($listOut -notmatch "Режим: list" -or $listOut -notmatch "Cloudflare WARP") {
    Write-Error "Вывод ping WARP -mode list не отформатирован в виде списка!"
}
Write-Host "    ✓ Режим list успешно активирован и проверен." -ForegroundColor Green

Write-Host "==> [E2E Test] 4. Проверка сортировки -sort slowest..." -ForegroundColor Cyan
$slowOut = & $binPath ping WARP -mode list -sort slowest --threads 4 | Out-String
if ($slowOut -notmatch "Сортировка: slowest") {
    Write-Error "Сортировка slowest не указана в выводе!"
}
Write-Host "    ✓ Сортировка slowest успешно принята." -ForegroundColor Green

Write-Host "==> [E2E Test] 5. Проверка экспорта отчета --export..." -ForegroundColor Cyan
$exportFile = ".\tests\scratch_ping_report.json"
if (Test-Path $exportFile) { Remove-Item $exportFile -Force }

$exportOut = & $binPath ping WARP --threads 4 --export $exportFile | Out-String
if (-not (Test-Path $exportFile)) {
    Write-Error "Файл экспорта $exportFile не был создан!"
}

$jsonContent = Get-Content $exportFile -Raw | ConvertFrom-Json
if ($null -eq $jsonContent.total -or $null -eq $jsonContent.results) {
    Write-Error "JSON отчет поврежден или не содержит обязательных полей (total, results)!"
}
Write-Host "    ✓ Отчет успешно создан, валиден: проверено $($jsonContent.total) узлов, средний пинг: $($jsonContent.average_ms) ms." -ForegroundColor Green

Write-Host "==> [E2E Test] 6. Проверка флага --no-save..." -ForegroundColor Cyan
$noSaveOut = & $binPath ping WARP --no-save --threads 4 | Out-String
if ($noSaveOut -notmatch "Сохранение результатов в LocalStorage отключено") {
    Write-Error "Флаг --no-save не отключил сохранение!"
}
Write-Host "    ✓ Флаг --no-save корректно обработан." -ForegroundColor Green

Write-Host "==> [E2E Test] 7. Проверка бэкапа и окна подтверждения recovery в режиме merge..." -ForegroundColor Cyan
$backupName = "e2e_merge_test"
$backupOut = & $binPath backup $backupName .\backups true true true | Out-String
$backupFile = ".\backups\$backupName.json"
if (-not (Test-Path $backupFile)) {
    Write-Error "Файл бэкапа $backupFile не был создан!"
}

$recoveryOut = & $binPath recovery $backupFile true true true -m merge -y | Out-String
if ($recoveryOut -notmatch "ПОДТВЕРЖДЕНИЕ ВОССТАНОВЛЕНИЯ" -or $recoveryOut -notmatch "MERGE") {
    Write-Error "Окно подтверждения recovery или режим MERGE не отобразились!"
}
Write-Host "    ✓ Окно подтверждения recovery и режим MERGE успешно отработали." -ForegroundColor Green

# Очистка временных файлов
if (Test-Path $exportFile) { Remove-Item $exportFile -Force }
if (Test-Path $backupFile) { Remove-Item $backupFile -Force }

Write-Host "============================================================" -ForegroundColor Green
Write-Host " ВСЕ E2E ТЕСТЫ (PING, ASSETS, RECOVERY MERGE) ПРОШЛИ УСПЕШНО! " -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
