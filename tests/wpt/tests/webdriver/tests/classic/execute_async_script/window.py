import pytest

from webdriver.client import WebFrame, WebWindow

from tests.support.asserts import assert_success
from . import execute_async_script


@pytest.mark.parametrize("expression, expected_type", [
    ("window.frames[0]", WebFrame),
    ("window", WebWindow),
], ids=["frame", "window"])
def test_web_reference(session, get_test_page, expression, expected_type):
    session.url = get_test_page()

    result = execute_async_script(session, f"arguments[0]({expression})")
    reference = assert_success(result)

    assert isinstance(reference, expected_type)

    if isinstance(reference, WebWindow):
        assert reference.id in session.handles
    else:
        assert reference.id not in session.handles


def test_window_open(session):
    result = execute_async_script(
        session, "window.foo = window.open(); arguments[0](window.foo);")
    reference = assert_success(result)

    assert isinstance(reference, WebWindow)
    assert reference.id in session.handles
