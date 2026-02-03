@echo off

set workdir=%~dp0
uv run --frozen python %workdir%mach %*
