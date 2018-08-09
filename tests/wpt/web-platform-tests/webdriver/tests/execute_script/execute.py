import pytest

from tests.support.asserts import assert_error, assert_success


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/sync".format(
            session_id=session.session_id),
        body)


def test_no_browsing_context(session, closed_window):
    response = execute_script(session, "return 1;")
    assert_error(response, "no such window")


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
