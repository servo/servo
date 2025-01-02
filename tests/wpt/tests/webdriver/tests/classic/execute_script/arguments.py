import pytest

from webdriver.client import ShadowRoot, WebElement, WebFrame, WebWindow

from tests.support.asserts import assert_error, assert_success
from . import execute_script


def test_null(session):
    value = None
    result = execute_script(session, "return [arguments[0] === null, arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] is True
    assert actual[1] == value


@pytest.mark.parametrize("value, expected_type", [
    (True, "boolean"),
    (42, "number"),
    ("foo", "string"),
], ids=["boolean", "number", "string"])
def test_primitives(session, value, expected_type):
    result = execute_script(session, "return [typeof arguments[0], arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] == expected_type
    assert actual[1] == value


def test_collection(session):
    value = [1, 2, 3]
    result = execute_script(session, "return [Array.isArray(arguments[0]), arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] is True
    assert actual[1] == value


def test_object(session):
    value = {"foo": "bar", "cheese": 23}
    result = execute_script(session, "return [typeof arguments[0], arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] == "object"
    assert actual[1] == value


def test_no_such_element_with_unknown_id(session):
    element = WebElement(session, "foo")

    result = execute_script(session, "return true;", args=[element])
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    result = execute_script(session, "return true;", args=[element])
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("div", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    result = execute_script(session, "return true;", args=[element])
    assert_error(result, "no such element")


def test_no_such_shadow_root_with_unknown_id(session):
    shadow_root = ShadowRoot(session, "foo")

    result = execute_script(session, "return true;", args=[shadow_root])
    assert_error(result, "no such shadow root")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_shadow_root_from_other_window_handle(session, get_test_page, closed):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)
    shadow_root = element.shadow_root

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    result = execute_script(session, "return true;", args=[shadow_root])
    assert_error(result, "no such shadow root")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_shadow_root_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("custom-element", all=False)
    shadow_root = element.shadow_root

    session.switch_frame("parent")

    if closed:
        execute_script(session, "arguments[0].remove();", args=[frame])

    result = execute_script(session, "return true;", args=[shadow_root])
    assert_error(result, "no such shadow root")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_detached_shadow_root_reference(session, stale_element, as_frame):
    shadow_root = stale_element("custom-element", as_frame=as_frame, want_shadow_root=True)

    result = execute_script(session, "return 1;", args=[shadow_root])
    assert_error(result, "detached shadow root")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    result = execute_script(session, "return 1;", args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("type", [WebFrame, WebWindow], ids=["frame", "window"])
@pytest.mark.parametrize("value", [None, False, 42, [], {}])
def test_invalid_argument_for_window_with_invalid_type(session, type, value):
    reference = type(session, value)

    result = execute_script(session, "return true", args=(reference,))
    assert_error(result, "invalid argument")


def test_no_such_window_for_window_with_invalid_value(session, get_test_page):
    session.url = get_test_page()

    result = execute_script(session, "return [window, window.frames[0]];")
    [window, frame] = assert_success(result)

    assert isinstance(window, WebWindow)
    assert isinstance(frame, WebFrame)

    window_reference = WebWindow(session, frame.id)
    frame_reference = WebFrame(session, window.id)

    for reference in [window_reference, frame_reference]:
        result = execute_script(session, "return true", args=(reference,))
        assert_error(result, "no such window")


@pytest.mark.parametrize("expression, expected_type", [
    ("window.frames[0]", WebFrame),
    ("document.querySelector('div')", WebElement),
    ("document.querySelector('custom-element').shadowRoot", ShadowRoot),
    ("window", WebWindow)
], ids=["frame", "node", "shadow-root", "window"])
def test_element_reference(session, get_test_page, expression, expected_type):
    session.url = get_test_page(as_frame=False)

    result = execute_script(session, f"return {expression}")
    reference = assert_success(result)
    assert isinstance(reference, expected_type)

    result = execute_script(session, f"return arguments[0] == {expression}", [reference])
    assert_success(result, True)
