import pytest

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


@pytest.mark.parametrize("expression, expected", [
    ("null", None),
    ("undefined", None),
    ("true", True),
    ("false", False),
    ("23", 23),
    ("'foo'", "foo"),
    (
        # Compute value in the runtime to reduce the potential for
        # interference from encoding literal bytes or escape sequences in
        # Python and HTTP.
        "String.fromCharCode(0)",
        "\x00"
    )
])
def test_primitive_serialization(session, expression, expected):
    response = execute_script(session, "return {};".format(expression))
    value = assert_success(response)
    assert value == expected


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
