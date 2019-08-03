import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/sync".format(
            session_id=session.session_id),
        body)


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/execute/sync".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_browsing_context(session, closed_window):
    response = execute_script(session, "return 1;")
    assert_error(response, "no such window")


def test_ending_comment(session):
    response = execute_script(session, "return 1; // foo")
    assert_success(response, 1)


def test_override_listeners(session):
    session.url = inline("""
<script>
called = [];
window.addEventListener = () => {called.push("Internal addEventListener")}
window.removeEventListener = () => {called.push("Internal removeEventListener")}
</script>
})""")
    response = execute_script(session, "return !window.onunload");
    assert_success(response, True);
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

    # The first alert has been accepted by the user prompt handler, the second one remains.
    # FIXME: this is how browsers currently work, but the spec should clarify if this is the
    #        expected behavior, see https://github.com/w3c/webdriver/issues/1153.
    assert session.alert.text == "Bye"

    session.alert.accept()
