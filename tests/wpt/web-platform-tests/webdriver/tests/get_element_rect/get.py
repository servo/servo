from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import element_rect
from tests.support.inline import inline


def get_element_rect(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/rect".format(
            session_id=session.session_id,
            element_id=element_id,
        )
    )


def test_no_top_browsing_context(session, closed_window):
    response = get_element_rect(session, "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_element_rect(session, "foo")
    assert_error(response, "no such window")


def test_element_not_found(session):
    result = get_element_rect(session, "foo")
    assert_error(result, "no such element")


def test_element_stale(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    session.refresh()

    result = get_element_rect(session, element.id)
    assert_error(result, "stale element reference")


def test_basic(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    result = get_element_rect(session, element.id)
    assert_success(result, element_rect(session, element))
