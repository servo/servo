from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def get_element_css_value(session, element_id, prop):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/css/{prop}".format(
            session_id=session.session_id,
            element_id=element_id,
            prop=prop
        )
    )


def test_no_top_browsing_context(session, closed_window):
    response = get_element_css_value(session, "foo", "bar")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_element_css_value(session, "foo", "bar")
    assert_error(response, "no such window")


def test_element_not_found(session):
    result = get_element_css_value(session, "foo", "display")
    assert_error(result, "no such element")


def test_element_stale(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    session.refresh()

    result = get_element_css_value(session, element.id, "display")
    assert_error(result, "stale element reference")


def test_property_name_value(session):
    session.url = inline("""<input style="display: block">""")
    element = session.find.css("input", all=False)

    result = get_element_css_value(session, element.id, "display")
    assert_success(result, "block")


def test_property_name_not_existent(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)

    result = get_element_css_value(session, element.id, "foo")
    assert_success(result, "")
