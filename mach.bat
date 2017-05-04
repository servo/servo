@echo off

IF EXIST "%ProgramFiles(x86)%" (
  set "ProgramFiles32=%ProgramFiles(x86)%"
) ELSE (
  set "ProgramFiles32=%ProgramFiles%"
)

set VC14VARS=%VS140COMNTOOLS%..\..\VC\vcvarsall.bat
IF EXIST "%VC14VARS%" (
  set "VS_VCVARS=%VC14VARS%"
) ELSE (
  for %%e in (Enterprise Professional Community) do (
    IF EXIST "%ProgramFiles32%\Microsoft Visual Studio\2017\%%e\VC\Auxiliary\Build\vcvarsall.bat" (
      set "VS_VCVARS=%ProgramFiles32%\Microsoft Visual Studio\2017\%%e\VC\Auxiliary\Build\vcvarsall.bat"
    )
  )
)

IF EXIST "%VS_VCVARS%" (
  IF NOT DEFINED Platform (
    IF EXIST "%ProgramFiles(x86)%" (
      call "%VS_VCVARS%" x64
    ) ELSE (
      ECHO 32-bit Windows is currently unsupported.
      EXIT /B
    )
  )
) ELSE (
  ECHO Visual Studio 2015 or 2017 is not installed.
  ECHO Download and install Visual Studio 2015 or 2017 from https://www.visualstudio.com/
  EXIT /B
)

python mach %*
