from tests.support.asserts import assert_error
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def get_window_rect(session):
    return session.transport.send(
        "GET", "session/{session_id}/window/rect".format(**vars(session)))


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()
    session.close()
    response = get_window_rect(session)
    assert_error(response, "no such window")


def test_payload(session):
    response = get_window_rect(session)

    assert response.status == 200
    assert isinstance(response.body["value"], dict)
    value = response.body["value"]
    expected = session.execute_script("""return {
         x: window.screenX,
         y: window.screenY,
         width: window.outerWidth,
         height: window.outerHeight
    }""")
    assert expected == value
