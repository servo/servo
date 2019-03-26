@echo off

setlocal

if EXIST "%VCINSTALLDIR%" (
  GOTO mach
)

pushd .

IF EXIST "%ProgramFiles(x86)%" (
  set "ProgramFiles32=%ProgramFiles(x86)%"
) ELSE (
  set "ProgramFiles32=%ProgramFiles%"
)

for %%v in (2019 2017) do (
  for %%e in (Enterprise Professional Community BuildTools) do (
    IF EXIST "%ProgramFiles32%\Microsoft Visual Studio\%%v\%%e\VC\Auxiliary\Build\vcvarsall.bat" (
      set "VS_VCVARS=%ProgramFiles32%\Microsoft Visual Studio\%%v\%%e\VC\Auxiliary\Build\vcvarsall.bat"
      GOTO vcvars
    )
  )
)

set VC14VARS=%VS140COMNTOOLS%..\..\VC\vcvarsall.bat
IF EXIST "%VC14VARS%" (
  set "VS_VCVARS=%VC14VARS%"
)

:vcvars
IF EXIST "%VS_VCVARS%" (
  IF NOT DEFINED Platform (
    IF EXIST "%ProgramFiles(x86)%" (
      call "%VS_VCVARS%" x64
    ) ELSE (
      ECHO 32-bit Windows is currently unsupported.
      GOTO bad_exit
    )
  )
) ELSE (
  ECHO Visual Studio 2015, 2017, or 2019 is not installed.
  ECHO Download and install Visual Studio from https://www.visualstudio.com/
  GOTO bad_exit
)

popd

:mach
where /Q py.exe
IF %ERRORLEVEL% NEQ 0 (
  python mach %*
) ELSE (
  py -2 mach %*
)

GOTO exit

:bad_exit
endlocal
EXIT /B 1

:exit
endlocal
exit /B
