REM skip "coveralls" run in PRs or forks
if "%TOXENV%" == "coveralls" (
    if not defined COVERALLS_REPO_TOKEN (
        echo skipping coveralls run because COVERALLS_REPO_TOKEN is not defined
        exit /b 0
    )
)
C:\Python36\python -m tox
