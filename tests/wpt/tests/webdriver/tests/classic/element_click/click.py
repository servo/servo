import pytest
from webdriver import WebElement

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
    element = WebElement(session, "foo")
    response = element_click(session, element)
    assert_error(response, "no such window")

    original_handle, element = closed_window
    response = element_click(session, element)
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = element_click(session, element)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    element = WebElement(session, "foo")

    response = element_click(session, element)
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = element_click(session, element)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = element_click(session, element.shadow_root)
    assert_error(result, "no such element")


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
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("input#text", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = element_click(session, element)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    response = element_click(session, element)
    assert_error(response, "stale element reference")
