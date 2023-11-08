import pytest

from webdriver import WebElement

from tests.support.asserts import assert_error, assert_same_element, assert_success


def get_shadow_root(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/shadow".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_shadow_root(session, element.id)
    assert_error(response, "no such window")
    response = get_shadow_root(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_shadow_root(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("div", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("custom-element", as_frame=as_frame)

    result = get_shadow_root(session, element.id)
    assert_error(result, "stale element reference")


def test_get_shadow_root(session, get_test_page):
    session.url = get_test_page()

    host_element = session.find.css("custom-element", all=False)

    response = get_shadow_root(session, host_element.id)
    value = assert_success(response)
    assert isinstance(value, dict)
    assert "shadow-6066-11e4-a52e-4f735466cecf" in value

    expected_host = session.execute_script("""
        return arguments[0].shadowRoot.host
        """, args=(host_element,))

    assert_same_element(session, host_element, expected_host)


def test_no_shadow_root(session, inline):
    session.url = inline("<div><p>no shadow root</p></div>")
    element = session.find.css("div", all=False)
    response = get_shadow_root(session, element.id)
    assert_error(response, "no such shadow root")
