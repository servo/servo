; Inno Setup Script for Servo Browser (for Windows)
; Produces a single standalone setup executable: Setup_ServoBrowser_v0.7.0.exe

#define MyAppName "Servo Browser"
#define MyAppVersion "0.7.0"
#define MyAppPublisher "The Servo Project Developers"
#define MyAppURL "https://servo.org"
#define MyAppExeName "servobrowser.exe"

[Setup]
AppId={{9B78D8D3-6052-409B-A162-4C10887A84B9}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\..\..\target\installer
OutputBaseFilename=Setup_ServoBrowser_v{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Check: FileExists('..\..\..\target\release\' + '{#MyAppExeName}')
Source: "..\..\..\target\debug\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Check: not FileExists('..\..\..\target\release\' + '{#MyAppExeName}')
Source: "..\..\..\target\release\libEGL.dll"; DestDir: "{app}"; Flags: ignoreversion; Check: FileExists('..\..\..\target\release\libEGL.dll')
Source: "..\..\..\target\debug\libEGL.dll"; DestDir: "{app}"; Flags: ignoreversion; Check: not FileExists('..\..\..\target\release\libEGL.dll')
Source: "..\..\..\target\release\libGLESv2.dll"; DestDir: "{app}"; Flags: ignoreversion; Check: FileExists('..\..\..\target\release\libGLESv2.dll')
Source: "..\..\..\target\debug\libGLESv2.dll"; DestDir: "{app}"; Flags: ignoreversion; Check: not FileExists('..\..\..\target\release\libGLESv2.dll')
Source: "..\..\..\resources\*"; DestDir: "{app}\resources"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; App Paths for Win+R 'servobrowser'
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\{#MyAppExeName}"; ValueType: string; ValueName: "Path"; ValueData: "{app}"

; Default Browser Capabilities (StartMenuInternet)
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser"; ValueType: string; ValueName: ""; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"""
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#MyAppName}"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Servo Browser is a fast, memory-safe, lightweight web browser."
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{app}\{#MyAppExeName},0"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\Capabilities\URLAssociations"; ValueType: string; ValueName: "http"; ValueData: "ServoBrowserHTML"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\ServoBrowser\Capabilities\URLAssociations"; ValueType: string; ValueName: "https"; ValueData: "ServoBrowserHTML"
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "ServoBrowser"; ValueData: "Software\Clients\StartMenuInternet\ServoBrowser\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
