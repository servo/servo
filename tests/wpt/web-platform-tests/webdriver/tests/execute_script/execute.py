import pytest

from webdriver import Element
from webdriver.error import NoSuchAlertException
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.sync import Poll
from . import execute_script


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/execute/sync".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = execute_script(session, "return 1;")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = execute_script(session, "return 1;")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = Element("foo", session)

    result = execute_script(session, "return true;", args=[element])
    assert_error(result, "no such element")


def test_no_such_element_from_other_window_handle(session, inline):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()
    session.window_handle = new_handle

    result = execute_script(session, "return true;", args=[element])
    assert_error(result, "no such element")


def test_no_such_element_from_other_frame(session, iframe, inline):
    session.url = inline(iframe("<div id='parent'><p/>"))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("#parent", all=False)
    session.switch_frame("parent")

    result = execute_script(session, "return true;", args=[element])
    assert_error(result, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference_as_argument(session, stale_element, as_frame):
    element = stale_element("<div>", "div", as_frame=as_frame)

    result = execute_script(session, "return 1;", args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference_as_returned_value(session, iframe, inline, as_frame):
    if as_frame:
        session.url = inline(iframe("<div>"))
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)
    else:
        session.url = inline("<div>")

    element = session.find.css("div", all=False)

    result = execute_script(session, """
        const elem = arguments[0];
        elem.remove();
        return elem;
        """, args=[element])
    assert_error(result, "stale element reference")


def test_opening_new_window_keeps_current_window_handle(session, inline):
    original_handle = session.window_handle
    original_handles = session.handles

    url = inline("""<a href="javascript:window.open();">open window</a>""")
    session.url = url
    session.find.css("a", all=False).click()
    wait = Poll(
        session,
        timeout=5,
        message="No new window has been opened")
    new_handles = wait.until(lambda s: set(s.handles) - set(original_handles))

    assert len(new_handles) == 1
    assert session.window_handle == original_handle
    assert session.url == url


def test_ending_comment(session):
    response = execute_script(session, "return 1; // foo")
    assert_success(response, 1)


def test_override_listeners(session, inline):
    session.url = inline("""
<script>
called = [];
window.addEventListener = () => {called.push("Internal addEventListener")}
window.removeEventListener = () => {called.push("Internal removeEventListener")}
</script>
})""")
    response = execute_script(session, "return !window.onunload")
    assert_success(response, True)
    response = execute_script(session, "return called")
    assert_success(response, [])


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_abort_by_user_prompt(session, dialog_type):
    response = execute_script(
        session, "window.{}('Hello'); return 1;".format(dialog_type))
    assert_success(response, None)

    session.alert.accept()


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_abort_by_user_prompt_twice(session, dialog_type):
    response = execute_script(
        session, "window.{0}('Hello'); window.{0}('Bye'); return 1;".format(dialog_type))
    assert_success(response, None)

    session.alert.accept()

    # The first alert has been accepted by the user prompt handler, the second
    # alert will still be opened because the current step isn't aborted.
    wait = Poll(
        session,
        timeout=5,
        message="Second alert has not been opened",
        ignored_exceptions=NoSuchAlertException
    )
    text = wait.until(lambda s: s.alert.text)

    assert text == "Bye"

    session.alert.accept()
