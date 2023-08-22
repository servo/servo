# mypy: allow-untyped-defs

import sys
from os.path import dirname, join
from unittest import mock

import pytest

sys.path.insert(0, join(dirname(__file__), "..", "..", ".."))

from mozlog import structuredlog

from wptrunner.browsers import base


class MozLogTestHandler(object):
    def __init__(self):
        self.items = []

    def __call__(self, data):
        self.items.append(data)


@pytest.mark.skipif(
    sys.platform == "win32",
    reason="Relies on echo, which isn't an executable on Windows",
)
def test_logging_immediate_exit():
    logger = structuredlog.StructuredLogger("test")
    handler = MozLogTestHandler()
    logger.add_handler(handler)

    class CustomException(Exception):
        pass

    with mock.patch.object(base, "wait_for_service", side_effect=CustomException):
        browser = base.WebDriverBrowser(
            logger, webdriver_binary="echo", webdriver_args=["sample output"]
        )
        try:
            with pytest.raises(CustomException):
                browser.start(group_metadata={})
        finally:
            # Ensure the `echo` process actually exits
            browser._proc.wait()

    process_output_actions = [
        data for data in handler.items if data["action"] == "process_output"
    ]

    assert len(process_output_actions) == 1
    assert process_output_actions[0]["data"] == "sample output"
