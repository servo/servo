# META: timeout=long

import pytest
import webdriver.protocol as protocol

from tests.support.asserts import assert_error, assert_success


PROMPT_HANDLERS = [
    "accept",
    "accept and notify",
    "dismiss",
    "dismiss and notify",
    "ignore",
]

PROMPT_TYPES = [
    "alert",
    "beforeUnload",
    "confirm",
    "default",
    "prompt",
]


def element_send_keys(session, element, text):
    return session["transport"].send(
        "POST",
        f"/session/{session['sessionId']}/element/{element.id}/value",
        {"text": text},
    )


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session["transport"].send(
        "POST",
        f"/session/{session['sessionId']}/execute/sync",
        body,
        encoder=protocol.Encoder,
        decoder=protocol.Decoder,
        session=session,
    )


def accept_alert(session):
    return session["transport"].send(
        "POST", f"session/{session['sessionId']}/alert/accept"
    )


def navigate_to(session, url):
    return session["transport"].send(
        "POST", f"session/{session['sessionId']}/url", {"url": url}
    )


def test_unhandled_prompt_behavior_as_string_default(
    new_session, add_browser_capabilities
):
    response, _ = new_session(
        {"capabilities": {"alwaysMatch": add_browser_capabilities({})}}
    )
    value = assert_success(response)
    assert value["capabilities"]["unhandledPromptBehavior"] == "dismiss and notify"


@pytest.mark.parametrize("handler", PROMPT_HANDLERS)
def test_unhandled_prompt_behavior_as_string(
    new_session, add_browser_capabilities, handler
):
    response, _ = new_session(
        {
            "capabilities": {
                "alwaysMatch": add_browser_capabilities(
                    {"unhandledPromptBehavior": handler}
                )
            }
        }
    )
    value = assert_success(response)
    assert value["capabilities"]["unhandledPromptBehavior"] == handler


@pytest.mark.parametrize(
    "handler,expected_capability,closed,notify",
    [
        (  # Check the default behavior with no handlers defined
            {},
            {},
            True,
            True,
        ),
        (  # Check the default behavior with a custom value defined
            {"default": "accept"},
            {"default": "accept"},
            True,
            False,
        ),
        (  # Check the default behavior with a custom value and override defined
            {"default": "accept", "alert": "ignore"},
            {"default": "accept", "alert": "ignore"},
            False,
            True,
        ),
    ],
)
def test_unhandled_prompt_behavior_as_object_default(
    new_session, add_browser_capabilities, handler, expected_capability, closed, notify
):
    response, session = new_session(
        {
            "capabilities": {
                "alwaysMatch": add_browser_capabilities(
                    {"unhandledPromptBehavior": handler}
                )
            }
        }
    )
    value = assert_success(response)
    assert value["capabilities"]["unhandledPromptBehavior"] == expected_capability

    execute_script(session, "alert('foo');")

    # Open user prompt and check if an error gets reported if expected
    response = execute_script(session, "window.result = 1; return window.result;")
    if notify:
        assert_error(response, "unexpected alert open")
    else:
        assert_success(response, 1)

    # Check that the user prompt has already closed if one was expected
    response = accept_alert(session)
    if closed:
        assert_error(response, "no such alert")
    else:
        assert_success(response, None)


@pytest.mark.parametrize("handler", PROMPT_HANDLERS)
@pytest.mark.parametrize("prompt", PROMPT_TYPES)
def test_unhandled_prompt_behavior_as_object(
    new_session, add_browser_capabilities, prompt, handler
):
    response, _ = new_session(
        {
            "capabilities": {
                "alwaysMatch": add_browser_capabilities(
                    {"unhandledPromptBehavior": {prompt: handler}}
                )
            }
        }
    )
    value = assert_success(response)
    assert value["capabilities"]["unhandledPromptBehavior"] == {prompt: handler}


@pytest.mark.parametrize("handler", PROMPT_HANDLERS)
def test_beforeunload_prompts_always_automatically_accepted(
    new_session, add_browser_capabilities, url, handler
):
    response, session = new_session(
        {
            "capabilities": {
                "alwaysMatch": add_browser_capabilities(
                    {"unhandledPromptBehavior": {"beforeUnload": handler}}
                )
            }
        }
    )
    assert_success(response)

    page_beforeunload = url("/webdriver/tests/support/html/beforeunload.html")
    navigate_to(session, page_beforeunload)

    response = execute_script(
        session,
        """
        return document.querySelector("input");
    """,
    )
    elem = assert_success(response)

    # Set sticky user activation
    response = element_send_keys(session, elem, "foo")
    assert_success(response)

    # Trigger a beforeunload prompt
    response = navigate_to(session, "about:blank")
    assert_success(response)

    # Prompt was automatically accepted
    response = accept_alert(session)
    assert_error(response, "no such alert")
