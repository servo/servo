@echo off
reg add "HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppContainer\Storage\microsoft.microsoftedge_8wekyb3d8bbwe\MicrosoftEdge\New Windows" /v "PopupMgr" /t REG_SZ /d no


REM Download and install the Ahem font
REM - https://wiki.saucelabs.com/display/DOCS/Downloading+Files+to+a+Sauce+Labs+Virtual+Machine+Prior+to+Testing
REM - https://superuser.com/questions/201896/how-do-i-install-a-font-from-the-windows-command-prompt
bitsadmin.exe /transfer "JobName" https://github.com/w3c/web-platform-tests/raw/master/fonts/Ahem.ttf "%WINDIR%\Fonts\Ahem.ttf"
reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts" /v "Ahem (TrueType)" /t REG_SZ /d Ahem.ttf /f
