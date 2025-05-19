import asyncio
import pytest
import pytest_asyncio
from webdriver.error import NoSuchAlertException

from ..bidi import (
    any_string,
    recursive_compare,
)

pytestmark = pytest.mark.asyncio

USER_PROMPT_CLOSED_EVENT = "browsingContext.userPromptClosed"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest_asyncio.fixture
async def check_beforeunload_implicitly_accepted(
    bidi_session,
    current_session,
    setup_beforeunload_page,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    new_tab,
    execute_as_async,
    url,
):
    async def check_beforeunload_implicitly_accepted():
        current_session.window_handle = new_tab["context"]

        page_beforeunload = await setup_beforeunload_page(new_tab)
        page_target = url("/webdriver/tests/support/html/default.html")

        on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)
        on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

        await subscribe_events([USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

        # Using WebDriver classic's navigation command to navigate away from
        # the page can hang if the prompt is not accepted. As such, wrap the
        # command to fail on timeout gracefully.
        def sync_navigate():
            current_session.url = page_target

        await wait_for_future_safe(
            asyncio.create_task(execute_as_async(sync_navigate)))

        # Wait for BiDi events.
        opened_event = await wait_for_future_safe(on_prompt_opened)
        recursive_compare(
            {
                "context": new_tab["context"],
                "type": "beforeunload",
                "message": any_string,
            },
            opened_event,
        )

        closed_event = await wait_for_future_safe(on_prompt_closed)
        recursive_compare(
            {
                "accepted": True,
                "context": new_tab["context"],
                "type": "beforeunload",
            },
            closed_event,
        )

        # Assert via classic that the alert is not present.
        with pytest.raises(NoSuchAlertException):
            current_session.alert.text

        # Assert the classic url changed.
        assert current_session.url == page_target

    return check_beforeunload_implicitly_accepted


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
async def test_accept(check_beforeunload_implicitly_accepted):
    await check_beforeunload_implicitly_accepted()


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
async def test_accept_and_notify(check_beforeunload_implicitly_accepted):
    await check_beforeunload_implicitly_accepted()


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
async def test_dismiss(check_beforeunload_implicitly_accepted):
    await check_beforeunload_implicitly_accepted()


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
async def test_dismiss_and_notify(check_beforeunload_implicitly_accepted):
    await check_beforeunload_implicitly_accepted()


@pytest.mark.capabilities({"unhandledPromptBehavior": "ignore"})
async def test_ignore(check_beforeunload_implicitly_accepted):
    await check_beforeunload_implicitly_accepted()
