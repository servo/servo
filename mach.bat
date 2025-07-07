@echo off

set workdir=%~dp0
uv run --no-project python %workdir%mach %*
