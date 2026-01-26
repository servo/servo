@echo off

set workdir=%~dp0
uv run --locked python %workdir%mach %*
