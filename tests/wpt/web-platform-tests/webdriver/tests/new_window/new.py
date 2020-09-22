import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success


def new_window(session, type_hint=None):
    return session.transport.send(
        "POST", "session/{session_id}/window/new".format(**vars(session)),
        {"type": type_hint})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/window/new".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = new_window(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    original_handles = session.handles

    response = new_window(session)
    value = assert_success(response)
    handles = session.handles
    assert len(handles) == len(original_handles) + 1
    assert value["handle"] in handles
    assert value["handle"] not in original_handles
    assert value["type"] in ["tab", "window"]


@pytest.mark.parametrize("type_hint", [True, 42, 4.2, [], {}])
def test_type_with_invalid_type(session, type_hint):
    response = new_window(session, type_hint)
    assert_error(response, "invalid argument")


def test_type_with_null_value(session):
    original_handles = session.handles

    response = new_window(session, type_hint=None)
    value = assert_success(response)
    handles = session.handles
    assert len(handles) == len(original_handles) + 1
    assert value["handle"] in handles
    assert value["handle"] not in original_handles
    assert value["type"] in ["tab", "window"]


def test_type_with_unknown_value(session):
    original_handles = session.handles

    response = new_window(session, type_hint="foo")
    value = assert_success(response)
    handles = session.handles
    assert len(handles) == len(original_handles) + 1
    assert value["handle"] in handles
    assert value["handle"] not in original_handles
    assert value["type"] in ["tab", "window"]
