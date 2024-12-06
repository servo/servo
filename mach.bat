@echo off
set workdir=%~dp0

where /Q py.exe
IF %ERRORLEVEL% NEQ 0 (
  python %workdir%mach %*
) ELSE (
  py -3 %workdir%mach %*
)
