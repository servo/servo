from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def get_window_rect(session):
    return session.transport.send(
        "GET", "session/{session_id}/window/rect".format(**vars(session)))


def test_no_browsing_context(session, closed_window):
    response = get_window_rect(session)
    assert_error(response, "no such window")


def test_payload(session):
    expected = session.execute_script("""return {
         x: window.screenX,
         y: window.screenY,
         width: window.outerWidth,
         height: window.outerHeight
    }""")

    response = get_window_rect(session)
    value = assert_success(response)

    assert isinstance(value, dict)
    assert value == expected
