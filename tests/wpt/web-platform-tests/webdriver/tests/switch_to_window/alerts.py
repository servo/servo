import pytest

from webdriver import error

from tests.support.asserts import assert_success
from tests.support.inline import inline


def switch_to_window(session, handle):
    return session.transport.send(
        "POST", "session/{session_id}/window".format(**vars(session)),
        {"handle": handle})


def test_retain_tab_modal_status(session, create_window):
    handle = session.window_handle

    new_handle = create_window()
    response = switch_to_window(session, new_handle)
    assert_success(response)

    session.url = inline("<script>window.alert('Hello');</script>")
    assert session.alert.text == "Hello"
    response = switch_to_window(session, handle)
    assert_success(response)

    with pytest.raises(error.NoSuchAlertException):
        session.alert.text == "Hello"
    response = switch_to_window(session, new_handle)
    assert_success(response)

    assert session.alert.text == "Hello"
