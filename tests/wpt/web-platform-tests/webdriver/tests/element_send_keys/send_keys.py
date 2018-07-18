import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def test_null_response_value(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    value = assert_success(response)
    assert value is None


@pytest.mark.parametrize("value", [True, None, 1, [], {}])
def test_invalid_text_type(session, value):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, value)
    assert_error(response, "invalid argument")


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()

    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    session.close()

    response = element_send_keys(session, element, "foo")
    assert_error(response, "no such window")


def test_stale_element(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    session.refresh()

    response = element_send_keys(session, element, "foo")
    assert_error(response, "stale element reference")
