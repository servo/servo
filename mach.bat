@echo off

SET VS_VCVARS=%VS140COMNTOOLS%..\..\VC\vcvarsall.bat
IF EXIST "%VS_VCVARS%" (
  IF NOT DEFINED VisualStudioVersion (
    IF EXIST "%ProgramFiles(x86)%" (
      call "%VS_VCVARS%" x64
    ) ELSE (
      call "%VS_VCVARS%" x86
    )
  )
) ELSE (
  ECHO Visual Studio 2015 is not installed.
  ECHO Download and install Visual Studio 2015 from https://www.visualstudio.com/
  EXIT /B
)

python mach %*
