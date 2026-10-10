#ifndef Arch
#define Arch "x64"
#endif
#define PluginSource "..\\..\\dist\\plugin"
#define HelperSource "..\\..\\dist\\windows\\mendeley-loopback-server.exe"

[Setup]
AppId={{C15DDE2A-B8F5-4E6C-884B-11D76B9A67D0}
AppName=Mendeley for ONLYOFFICE
AppVersion=1.1.4
DefaultDirName={localappdata}\Programs\MendeleyOnlyOffice
PrivilegesRequired=lowest
OutputDir=..\..\dist\installers
OutputBaseFilename=Mendeley-ONLYOFFICE-Setup-{#Arch}
ArchitecturesInstallIn64BitMode=x64 arm64
WizardStyle=modern
UninstallDisplayName=Mendeley for ONLYOFFICE

[Files]
Source: "{#HelperSource}"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "{#PluginSource}\*"; DestDir: "{code:GetPluginDir}"; Flags: ignoreversion recursesubdirs createallsubdirs

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\mendeley-onlyoffice"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "MendeleyOnlyOfficeHelper"; ValueData: """{app}\bin\mendeley-loopback-server.exe"""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\MendeleyOnlyOffice"; ValueType: string; ValueName: "PluginDir"; ValueData: "{code:GetPluginsParent}"; Flags: uninsdeletekey

[Run]
Filename: "{app}\bin\mendeley-loopback-server.exe"; Flags: runhidden nowait

[UninstallRun]
Filename: "{app}\bin\mendeley-loopback-server.exe"; Parameters: "--stop"; Flags: runhidden waituntilterminated skipifdoesntexist
[Code]
var
  PluginPage: TInputDirWizardPage;

procedure InitializeWizard;
var
  ExistingPluginDir: String;
begin
  PluginPage := CreateInputDirPage(wpSelectDir,
    'ONLYOFFICE plugin directory',
    'Choose ONLYOFFICE installation to add Mendeley to',
    'Select the sdkjs-plugins directory for your user installation.',
    False, 'sdkjs-plugins');
  PluginPage.Add('ONLYOFFICE plugin directory:');
  if RegQueryStringValue(HKCU, 'Software\MendeleyOnlyOffice', 'PluginDir', ExistingPluginDir) then
    PluginPage.Values[0] := ExistingPluginDir
  else
    PluginPage.Values[0] := ExpandConstant('{localappdata}\ONLYOFFICE\DesktopEditors\data\sdkjs-plugins');
end;

function GetPluginDir(Param: String): String;
begin
  Result := AddBackslash(PluginPage.Values[0]) + '{BE5CBF95-C0AD-4842-B157-AC40FEDD9441}';
end;

function GetPluginsParent(Param: String): String;
begin
  Result := PluginPage.Values[0];
end;


function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = PluginPage.ID then begin
    if not DirExists(PluginPage.Values[0]) then
      ForceDirectories(PluginPage.Values[0]);
  end;
end;
