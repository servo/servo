from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def get_element_tag_name(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/name".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()
    session.close()

    result = get_element_tag_name(session, "foo")
    assert_error(result, "no such window")


def test_element_not_found(session):
    result = get_element_tag_name(session, "foo")
    assert_error(result, "no such element")


def test_element_stale(session):
    session.url = inline("<input id=foo>")
    element = session.find.css("input", all=False)
    session.refresh()

    result = get_element_tag_name(session, element.id)
    assert_error(result, "stale element reference")


def test_get_element_tag_name(session):
    session.url = inline("<input id=foo>")
    element = session.find.css("input", all=False)

    result = get_element_tag_name(session, element.id)
    assert_success(result, "input")
