from io import StringIO

import pytest

from mozlog.formatters import ErrorSummaryFormatter, MachFormatter
from mozlog.handlers import StreamHandler
from mozlog.structuredlog import StructuredLogger


@pytest.fixture
def get_logger():
    # Ensure a new state instance is created for each test function.
    StructuredLogger._logger_states = {}
    formatters = {
        "mach": MachFormatter,
        "errorsummary": ErrorSummaryFormatter,
    }

    def inner(name, **fmt_args):
        buf = StringIO()
        fmt = formatters[name](**fmt_args)
        logger = StructuredLogger("test_logger")
        logger.add_handler(StreamHandler(buf, fmt))
        return logger

    return inner
