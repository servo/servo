import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success
from tests.support.inline import inline


def get_title(session):
    return session.transport.send(
        "GET", "session/{session_id}/title".format(**vars(session)))


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_accept(session, create_dialog, dialog_type):
    session.url = inline("<title>WD doc title</title>")
    expected_title = session.title

    create_dialog(dialog_type, text="dialog")

    response = get_title(session)
    assert_success(response, expected_title)

    assert_dialog_handled(session, expected_text="dialog")


def test_handle_prompt_accept_and_notify():
    """TODO"""


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_dismiss(session, create_dialog, dialog_type):
    session.url = inline("<title>WD doc title</title>")
    expected_title = session.title

    create_dialog(dialog_type, text="dialog")

    response = get_title(session)
    assert_success(response, expected_title)

    assert_dialog_handled(session, expected_text="dialog")


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_default(session, create_dialog, dialog_type):
    session.url = inline("<title>WD doc title</title>")

    create_dialog(dialog_type, text="dialog")

    response = get_title(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text="dialog")


# The behavior of the `window.print` function is platform-dependent and may not
# trigger the creation of a dialog at all. Therefore, this test should only be
# run in contexts that support the dialog (a condition that may not be
# determined automatically).
# def test_title_with_non_simple_dialog(session):
#    document = "<title>With non-simple dialog</title><h2>Hello</h2>"
#    spawn = """
#        var done = arguments[0];
#        setTimeout(function() {
#            done();
#        }, 0);
#        setTimeout(function() {
#            window['print']();
#        }, 0);
#    """
#    session.url = inline(document)
#    session.execute_async_script(spawn)
#
#    result = get_title(session)
#    assert_error(result, "unexpected alert open")
