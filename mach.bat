@echo off

REM UV defaults to x86_64 Python on Arm64, so we need to override that.
REM https://github.com/astral-sh/uv/issues/12906
if "%PROCESSOR_ARCHITECTURE%"=="ARM64" ( set UV_PYTHON=arm64 )

set workdir=%~dp0
uv run --frozen python %workdir%mach %*
