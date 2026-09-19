; NekoBoxPlusForPC Installer — InnoSetup Script
; Собирает установщик из release_dist/Windows/

#define MyAppName "NekoBoxPlusForPC"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "NekoBox Team"
#define MyAppURL "https://github.com/starifly/NekoBoxForAndroid"
#define MyAppExeName "nbpfpc.exe"

[Setup]
AppId={{B7E3C4D1-2F8A-4E9B-A3D6-1F2E3C4D5E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
; Права администратора не нужны для установки (portable-friendly)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\release_dist
OutputBaseFilename=NekoBoxPlusForPC-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
; Минимальные требования — Windows 7 SP1 x64
MinVersion=6.1sp1
ArchitecturesInstallIn64BitMode=x64
ArchitecturesAllowed=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "addtopath"; Description: "Добавить nbpfpc в системный PATH (рекомендуется)"; GroupDescription: "Дополнительно:"; Flags: checkedonce
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Основные бинарники
Source: "..\release_dist\Windows\nbpfpc.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release_dist\Windows\singbox.exe"; DestDir: "{app}"; Flags: ignoreversion

; Веб-панель Metacubexd
Source: "..\release_dist\Windows\metacubexd\*"; DestDir: "{app}\metacubexd"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName} CLI"; Filename: "{cmd}"; Parameters: "/k ""{app}\{#MyAppExeName}"" help"; WorkingDir: "{app}"
Name: "{group}\Открыть папку {#MyAppName}"; Filename: "{app}"
Name: "{group}\Удалить {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{cmd}"; Parameters: "/k ""{app}\{#MyAppExeName}"" help"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
; Добавление в PATH при выборе соответствующей опции
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath(ExpandConstant('{app}'))

[Run]
Filename: "{app}\{#MyAppExeName}"; Parameters: "help"; Description: "Запустить справку nbpfpc"; Flags: nowait postinstall skipifsilent shellexec

[UninstallRun]
; Остановка ядра при удалении (если запущено)
Filename: "taskkill"; Parameters: "/f /im singbox.exe"; Flags: runhidden; RunOnceId: "StopSingbox"
Filename: "taskkill"; Parameters: "/f /im nbpfpc.exe"; Flags: runhidden; RunOnceId: "StopNbpfpc"

[Code]
/// Проверяет, содержит ли PATH уже указанный каталог.
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER,
    'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;
