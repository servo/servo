from collections.abc import Generator

import pytest
from mozlog.structuredlog import StructuredLogger

from ..wptrunner import GlobalLogger


@pytest.fixture(scope="module")
def logger() -> Generator[StructuredLogger]:
    with GlobalLogger({}, {}) as _logger:
        yield _logger
