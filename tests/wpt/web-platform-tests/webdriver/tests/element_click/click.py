import pytest
from webdriver import Element

from tests.support.asserts import assert_error, assert_success


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_null_response_value(session, inline):
    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)

    response = element_click(session, element)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    element = Element("foo", session)
    response = element_click(session, element)
    assert_error(response, "no such window")

    original_handle, element = closed_window
    response = element_click(session, element)
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = element_click(session, element)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    element = Element("foo", session)

    response = element_click(session, element)
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = Element("foo", session)

    response = element_click(session, element)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = element_click(session, element)
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

    response = element_click(session, button)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("<div>", "div", as_frame=as_frame)

    response = element_click(session, element)
    assert_error(response, "stale element reference")
