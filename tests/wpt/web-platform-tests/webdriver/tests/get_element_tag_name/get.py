import pytest

from webdriver import Element

from tests.support.asserts import assert_error, assert_success


def get_element_tag_name(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/name".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such window")
    response = get_element_tag_name(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_tag_name(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = Element("foo", session)

    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_from_other_window_handle(session, inline):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_from_other_frame(session, iframe, inline):
    session.url = inline(iframe("<div id='parent'><p/>"))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("#parent", all=False)
    session.switch_frame("parent")

    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("<input>", "input", as_frame=as_frame)

    result = get_element_tag_name(session, element.id)
    assert_error(result, "stale element reference")


def test_get_element_tag_name(session, inline):
    session.url = inline("<input id=foo>")
    element = session.find.css("input", all=False)

    result = get_element_tag_name(session, element.id)
    assert_success(result, "input")
