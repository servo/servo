@echo off
set FONTCONFIG_FILE=C:\Program Files\Mozilla Research\Servo Tech Demo\fonts.conf
servo.exe -w --pref dom.mozbrowser.enabled --pref shell.builtin-key-shortcuts.enabled=false browserhtml\index.html
