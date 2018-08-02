# META: timeout=long

import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success
from tests.support.inline import inline


@pytest.fixture
def pages(session):
    pages = [
        inline("<p id=1>"),
        inline("<p id=2>"),
    ]

    for page in pages:
        session.url = page

    session.back()

    return pages


def forward(session):
    return session.transport.send(
        "POST", "session/{session_id}/forward".format(**vars(session)))


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_accept(session, create_dialog, dialog_type, pages):
    create_dialog(dialog_type, text=dialog_type)

    response = forward(session)
    assert_success(response)

    # retval not testable for confirm and prompt because window is gone
    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=None)

    assert session.url == pages[1]


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept_and_notify(session, create_dialog, dialog_type, retval, pages):
    create_dialog(dialog_type, text=dialog_type)

    response = forward(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.url == pages[0]


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_dismiss(session, create_dialog, dialog_type, pages):
    create_dialog(dialog_type, text=dialog_type)

    response = forward(session)
    assert_success(response)

    # retval not testable for confirm and prompt because window is gone
    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=None)

    assert session.url == pages[1]


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_dissmiss_and_notify(session, create_dialog, dialog_type, retval, pages):
    create_dialog(dialog_type, text=dialog_type)

    response = forward(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.url == pages[0]


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_default(session, create_dialog, dialog_type, retval, pages):
    create_dialog(dialog_type, text=dialog_type)

    response = forward(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.url == pages[0]
