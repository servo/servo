import pytest

from webdriver.client import WebFrame, WebWindow

from tests.support.asserts import assert_success
from . import execute_script


@pytest.mark.parametrize("expression, expected_type", [
    ("window.frames[0]", WebFrame),
    ("window", WebWindow),
], ids=["frame", "window"])
def test_web_reference(session, get_test_page, expression, expected_type):
    session.url = get_test_page()

    result = execute_script(session, f"return {expression}")
    reference = assert_success(result)

    assert isinstance(reference, expected_type)

    if isinstance(reference, WebWindow):
        assert reference.id in session.handles
    else:
        assert reference.id not in session.handles


@pytest.mark.parametrize("expression, expected_type", [
    ("window.frames[0]", WebFrame),
    ("window", WebWindow),
], ids=["frame", "window"])
def test_web_reference_in_array(session, get_test_page, expression, expected_type):
    session.url = get_test_page()

    result = execute_script(session, f"return [{expression}]")
    value = assert_success(result)

    assert isinstance(value[0], expected_type)

    if isinstance(value[0], WebWindow):
        assert value[0].id in session.handles
    else:
        assert value[0].id not in session.handles


@pytest.mark.parametrize("expression, expected_type", [
    ("window.frames[0]", WebFrame),
    ("window", WebWindow),
], ids=["frame", "window"])
def test_web_reference_in_object(session, get_test_page, expression, expected_type):
    session.url = get_test_page()

    result = execute_script(session, f"""return {{"ref": {expression}}}""")
    reference = assert_success(result)

    assert isinstance(reference["ref"], expected_type)

    if isinstance(reference["ref"], WebWindow):
        assert reference["ref"].id in session.handles
    else:
        assert reference["ref"].id not in session.handles


def test_window_open(session):
    result = execute_script(session, "window.foo = window.open(); return window.foo;")
    reference = assert_success(result)

    assert isinstance(reference, WebWindow)
    assert reference.id in session.handles


def test_same_id_after_cross_origin_navigation(session, get_test_page):
    params = {"pipe": "header(Cross-Origin-Opener-Policy,same-origin)"}

    first_page = get_test_page(parameters=params, protocol="https")
    second_page = get_test_page(parameters=params, protocol="https", domain="alt")

    session.url = first_page

    result = execute_script(session, "return window")
    window_before = assert_success(result)

    session.url = second_page

    result = execute_script(session, "return window")
    window_after = assert_success(result)

    assert window_before == window_after
