@echo off

set workdir=%~dp0
uv run python %workdir%mach %*
