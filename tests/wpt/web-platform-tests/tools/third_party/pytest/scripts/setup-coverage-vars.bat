if "%PYTEST_COVERAGE%" == "1" (
  set "_PYTEST_TOX_COVERAGE_RUN=coverage run -m"
  set "_PYTEST_TOX_EXTRA_DEP=coverage-enable-subprocess"
  echo Coverage vars configured, PYTEST_COVERAGE=%PYTEST_COVERAGE%
) else (
  echo Skipping coverage vars setup, PYTEST_COVERAGE=%PYTEST_COVERAGE%
)
