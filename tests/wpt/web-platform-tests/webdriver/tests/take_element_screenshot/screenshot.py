import base64
import imghdr

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def take_element_screenshot(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/screenshot".format(
            session_id=session.session_id,
            element_id=element_id,
        )
    )


def test_no_browsing_context(session, closed_window):
    response = take_element_screenshot(session, "foo")
    assert_error(response, "no such window")


def test_screenshot(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    response = take_element_screenshot(session, element.id)
    value = assert_success(response)

    image = base64.decodestring(value)
    assert imghdr.what("", image) == "png"


def test_stale(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    session.refresh()

    result = take_element_screenshot(session, element.id)
    assert_error(result, "stale element reference")
