import pytest

from webdriver.error import NoSuchElementException, NoSuchAlertException
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success


def switch_to_window(session, handle):
    return session.transport.send(
        "POST", "session/{session_id}/window".format(**vars(session)),
        {"handle": handle})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/window".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session):
    response = switch_to_window(session, session.new_window())
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session):
    original_handle = session.window_handle
    new_handle = session.new_window()

    session.window.close()
    assert original_handle not in session.handles, "Unable to close window"

    response = switch_to_window(session, new_handle)
    assert_success(response)

    assert session.window_handle == new_handle


def test_no_browsing_context(session, url):
    new_handle = session.new_window()

    session.url = url("/webdriver/tests/support/html/frames.html")
    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    deleteframe = session.find.css("#delete-frame", all=False)
    session.switch_frame(deleteframe)

    button = session.find.css("#remove-parent", all=False)
    button.click()

    response = switch_to_window(session, new_handle)
    assert_success(response)

    assert session.window_handle == new_handle


def test_switch_to_window_sets_top_level_context(session, inline, iframe):
    session.url = inline(iframe("<p>foo"))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = switch_to_window(session, session.window_handle)
    assert_success(response)

    session.find.css("iframe", all=False)


def test_element_not_found_after_tab_switch(session, inline):
    session.url = inline("<p id='a'>foo")
    paragraph = session.find.css("p", all=False)

    session.window_handle = session.new_window(type_hint="tab")

    with pytest.raises(NoSuchElementException):
        paragraph.attribute("id")


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_finds_exising_user_prompt_after_tab_switch(session, dialog_type):
    original_handle = session.window_handle
    new_handle = session.new_window()

    session.execute_script("{}('foo');".format(dialog_type))

    response = switch_to_window(session, new_handle)
    assert_success(response)

    with pytest.raises(NoSuchAlertException):
        session.alert.text

    session.window.close()

    response = switch_to_window(session, original_handle)
    assert_success(response)

    session.alert.accept()
