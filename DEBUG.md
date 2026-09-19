1. ✅ **Windows 7 x86 & data/ isolation** — `[RESOLVED]` 
   - Снята блокировка 32-bit: добавлены сборки под `i686-pc-windows-msvc` со статическим CRT (`+crt-static`) и Schannel (`--features win7-tls`).
   - Портабельность: все конфигурации, базы `geo/`, логи `logs/`, кэш `cache.db` и бэкапы изолированы в директории `data/` рядом с `nbpfpc.exe`.
   - Автономность: ядро `singbox.exe` и `metacubexd` размещаются прямо в папке сборки рядом с `nbpfpc.exe`.
   - Сборки автоматически пакуются в каталог `builds/` и `release_dist/`.

2. ✅ **Windows 7 x64 bcryptprimitives.dll** — `[RESOLVED]` Добавлена сборка Rust CLI с `--features win7-tls` (Schannel вместо rustls). Go runtime на Win7 — документировано как ограничение. Auto-detection Win7 + legacy binary selection в `CoreSupervisor`.

3. ✅ **Installer + Core Finder** — `[RESOLVED]`
   - Core Finder расширен: поиск в exe-dir, PATH, системных каталогах (LOCALAPPDATA, ProgramFiles, /usr/local/bin).
   - InnoSetup installer script: `installer/nekobox_setup.iss`.

4. ✅ **Builds WITH core inside** — `[RESOLVED]` `build_desktop.ps1` расширен:
   - Step 1: Go core build (Windows/Linux/MacOS/Windows7)
   - Step 2: Rust CLI build (normal + win7-tls)
   - Step 3: Distribution assembly + ZIP packaging

5. ✅ **Clash API / metacubexd** — `[RESOLVED]`
   - `external_ui` path теперь разрешается в абсолютный через `resolve_ui_path()` (ищет рядом с exe)
   - Добавлен `external_ui_download_url` для автозагрузки metacubexd если папка отсутствует
   - `config.json` обновлён с полными настройками clash_api
   - Добавлена диагностика `/version` + `/ui` endpoints после старта ядра