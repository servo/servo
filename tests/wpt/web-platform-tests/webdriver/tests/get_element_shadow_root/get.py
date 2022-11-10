import pytest

from webdriver import Element

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
    element = Element("foo", session)

    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_from_other_window_handle(session, inline, checkbox_dom):
    session.url = inline(checkbox_dom)
    element = session.find.css("custom-checkbox-element", all=False)

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_from_other_frame(session, iframe, inline, checkbox_dom):
    session.url = inline(iframe(checkbox_dom))

    session.switch_frame(0)
    element = session.find.css("custom-checkbox-element", all=False)
    session.switch_frame("parent")

    response = get_shadow_root(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, checkbox_dom, as_frame):
    element = stale_element(checkbox_dom, "custom-checkbox-element", as_frame=as_frame)

    result = get_shadow_root(session, element.id)
    assert_error(result, "stale element reference")


def test_get_shadow_root(session, inline, checkbox_dom):
    session.url = inline(checkbox_dom)
    expected = session.execute_script(
        "return document.querySelector('custom-checkbox-element').shadowRoot.host")
    custom_element = session.find.css("custom-checkbox-element", all=False)
    response = get_shadow_root(session, custom_element.id)
    value = assert_success(response)
    assert isinstance(value, dict)
    assert "shadow-6066-11e4-a52e-4f735466cecf" in value
    assert_same_element(session, custom_element, expected)


def test_no_shadow_root(session, inline):
    session.url = inline("<div><p>no shadow root</p></div>")
    element = session.find.css("div", all=False)
    response = get_shadow_root(session, element.id)
    assert_error(response, "no such shadow root")
