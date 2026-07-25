!include "MUI2.nsh"

Name "Fusion v2.0 Vortex"
OutFile "Fusion-v2.0-Vortex-Setup.exe"
InstallDir "$PROGRAMFILES\Fusion"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Fusion Compiler"
    SetOutPath $INSTDIR
    File "..\..\target\release\fuc.exe"
    File "..\..\target\release\fusion.exe"
    
    ; Install standard library
    SetOutPath $INSTDIR\stdlib
    File /r "..\..\stdlib\*.*"
    
    ; Add to PATH
    EnvironMB::AddValue "Path" "$INSTDIR"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Add to Programs menu
    CreateDirectory "$SMPROGRAMS\Fusion"
    CreateShortcut "$SMPROGRAMS\Fusion\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    CreateShortcut "$SMPROGRAMS\Fusion\Documentation.lnk" "$INSTDIR\docs\guides\ch01-getting-started.md"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\fuc.exe"
    Delete "$INSTDIR\fusion.exe"
    RMDir /r "$INSTDIR\stdlib"
    RMDir /r "$INSTDIR"
    EnvironMB::RemoveValue "Path" "$INSTDIR"
    Delete "$SMPROGRAMS\Fusion\Uninstall.lnk"
    Delete "$SMPROGRAMS\Fusion\Documentation.lnk"
    RMDir "$SMPROGRAMS\Fusion"
SectionEnd
