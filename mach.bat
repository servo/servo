@echo off

SET VS_VCVARS=%VS140COMNTOOLS%..\..\VC\vcvarsall.bat
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
  ECHO Visual Studio 2015 is not installed.
  ECHO Download and install Visual Studio 2015 from https://www.visualstudio.com/
  EXIT /B
)

python mach %*
