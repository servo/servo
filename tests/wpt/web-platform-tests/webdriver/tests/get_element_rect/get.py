import pytest

from webdriver import Element

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import element_rect


def get_element_rect(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/rect".format(
            session_id=session.session_id,
            element_id=element_id,
        )
    )


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_rect(session, element.id)
    assert_error(response, "no such window")
    response = get_element_rect(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_element_rect(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_rect(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = Element("foo", session)

    response = get_element_rect(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = get_element_rect(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, url, closed):
    session.url = url("/webdriver/tests/support/html/subframe.html")

    frame = session.find.css("#delete-frame", all=False)
    session.switch_frame(frame)

    button = session.find.css("#remove-parent", all=False)
    if closed:
        button.click()

    session.switch_frame("parent")

    response = get_element_rect(session, button.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("<input>", "input", as_frame=as_frame)

    result = get_element_rect(session, element.id)
    assert_error(result, "stale element reference")


def test_basic(session, inline):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    result = get_element_rect(session, element.id)
    assert_success(result, element_rect(session, element))
