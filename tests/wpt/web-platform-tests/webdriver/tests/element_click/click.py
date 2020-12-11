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
