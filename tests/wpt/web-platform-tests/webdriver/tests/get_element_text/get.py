import pytest

from webdriver import Element

from tests.support.asserts import assert_error, assert_success


def get_element_text(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/text".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_text(session, element.id)
    assert_error(response, "no such window")
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = Element("foo", session)

    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_from_other_window_handle(session, inline):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_from_other_frame(session, iframe, inline):
    session.url = inline(iframe("<div id='parent'><p/>"))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("#parent", all=False)
    session.switch_frame("parent")

    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("<input>", "input", as_frame=as_frame)

    response = get_element_text(session, element.id)
    assert_error(response, "stale element reference")


def test_getting_text_of_a_non_existant_element_is_an_error(session, inline):
    session.url = inline("""<body>Hello world</body>""")

    result = get_element_text(session, "foo")
    assert_error(result, "no such element")


def test_read_element_text(session, inline):
    session.url = inline("Before f<span id='id'>oo</span> after")
    element = session.find.css("#id", all=False)

    result = get_element_text(session, element.id)
    assert_success(result, "oo")


def test_pretty_print_xml(session, inline):
    session.url = inline("<xml><foo>che<bar>ese</bar></foo></xml>", doctype="xml")

    elem = session.find.css("foo", all=False)
    assert elem.text == "cheese"
