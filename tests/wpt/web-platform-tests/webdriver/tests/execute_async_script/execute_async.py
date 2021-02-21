import pytest

from webdriver.error import NoSuchAlertException
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.sync import Poll


def execute_async_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/async".format(**vars(session)),
        body)


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/execute/async".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = execute_async_script(session, "argument[0](1);")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = execute_async_script(session, "argument[0](1);")
    assert_error(response, "no such window")


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_abort_by_user_prompt(session, dialog_type):
    response = execute_async_script(
        session,
        "window.{}('Hello'); arguments[0](1);".format(dialog_type))
    assert_success(response, None)

    session.alert.accept()


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_abort_by_user_prompt_twice(session, dialog_type):
    response = execute_async_script(
        session,
        "window.{0}('Hello'); window.{0}('Bye'); arguments[0](1);".format(dialog_type))
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
