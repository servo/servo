REM script called by Azure to combine and upload coverage information to codecov
if "%PYTEST_COVERAGE%" == "1" (
    echo Prepare to upload coverage information
    if defined CODECOV_TOKEN (
        echo CODECOV_TOKEN defined
    ) else (
        echo CODECOV_TOKEN NOT defined
    )
    python -m pip install codecov
    python -m coverage combine
    python -m coverage xml
    python -m coverage report -m
    scripts\retry python -m codecov --required -X gcov pycov search -f coverage.xml --name %PYTEST_CODECOV_NAME%
) else (
    echo Skipping coverage upload, PYTEST_COVERAGE=%PYTEST_COVERAGE%
)
