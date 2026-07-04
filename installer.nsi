; Trix Audio Converter — NSIS Windows Installer
!include "MUI2.nsh"

Name "Trix Audio Converter"
OutFile "TrixAudioConverter-Setup.exe"
InstallDir "$PROGRAMFILES\TrixAudioConverter"
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!define MUI_UNABORTWARNING
!define MUI_ICON "assets\icons\icon.ico"
!define MUI_UNICON "assets\icons\icon.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Portuguese_Brazilian"
!insertmacro MUI_LANGUAGE "English"

Section "Trix Audio Converter" SecMain
    SetOutPath "$INSTDIR"
    
    ; Binário Rust
    File "src-rs\target\release\trix.exe"
    
    ; Frontend
    SetOutPath "$INSTDIR\dist"
    File /r "dist\*.*"
    
    ; Assets
    SetOutPath "$INSTDIR\assets"
    File /r "assets\*.*"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Registry (Add/Remove Programs)
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "DisplayName" "Trix Audio Converter"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "DisplayVersion" "1.0.0"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "Publisher" "Trix Audio Converter"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter" "NoRepair" 1
    
    ; Start Menu
    CreateDirectory "$SMPROGRAMS\Trix Audio Converter"
    CreateShortCut "$SMPROGRAMS\Trix Audio Converter\Trix Audio Converter.lnk" "$INSTDIR\trix.exe"
    CreateShortCut "$SMPROGRAMS\Trix Audio Converter\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    
    ; Desktop shortcut
    CreateShortCut "$DESKTOP\Trix Audio Converter.lnk" "$INSTDIR\trix.exe"
 SectionEnd
 
 Section "Uninstall"
     Delete "$INSTDIR\trix.exe"
     Delete "$INSTDIR\Uninstall.exe"
     RMDir /r "$INSTDIR\dist"
     RMDir /r "$INSTDIR\assets"
    RMDir "$INSTDIR"
    
    Delete "$SMPROGRAMS\Trix Audio Converter\Trix Audio Converter.lnk"
    Delete "$SMPROGRAMS\Trix Audio Converter\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Trix Audio Converter"
    Delete "$DESKTOP\Trix Audio Converter.lnk"
    
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\TrixAudioConverter"
SectionEnd
