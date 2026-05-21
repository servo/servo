import sys
from pathlib import Path

# Add webdriver to the Python path so we can import the fixtures
webdriver_tests_path = Path(__file__).parent.parent.parent / "webdriver"
sys.path.insert(0, str(webdriver_tests_path))

pytest_plugins = (
    "tests.support.fixtures",
    "tests.support.classic.fixtures",
    "aamtests.support.fixtures_a11y_api",
)
