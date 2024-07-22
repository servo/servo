import asyncio
import pytest
import pytest_asyncio
from webdriver.error import NoSuchAlertException

from tests.support.sync import AsyncPoll

from ..bidi import (
    any_string,
    recursive_compare,
)


pytestmark = pytest.mark.asyncio

USER_PROMPT_CLOSED_EVENT = "browsingContext.userPromptClosed"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest_asyncio.fixture
async def check_beforeunload_not_implicitly_accepted(
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
    async def check_beforeunload_not_implicitly_accepted(accept):
        current_session.window_handle = new_tab["context"]

        page_beforeunload = await setup_beforeunload_page(new_tab)
        page_target = url("/webdriver/tests/support/html/default.html")

        on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)
        on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

        await subscribe_events([USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

        # Using WebDriver classic's navigation command to navigate away from
        # the page will hang and wait for the beforeunload dialog to close.
        # As such start the command immediately as task but await for it later
        # when BiDi closed the prompt.
        def sync_navigate():
            current_session.url = page_target

        task_navigate = asyncio.create_task(execute_as_async(sync_navigate))
        opened_event = await wait_for_future_safe(on_prompt_opened)

        recursive_compare(
            {
                "context": new_tab["context"],
                "type": "beforeunload",
                "message": any_string,
            },
            opened_event,
        )

        # Close the beforeunload prompt and wait for the navigation to finish.
        await bidi_session.browsing_context.handle_user_prompt(
            context=new_tab["context"], accept=accept
        )
        closed_event = await wait_for_future_safe(on_prompt_closed)
        await task_navigate

        # Check that the beforeunload prompt is closed and the event was sent.
        with pytest.raises(NoSuchAlertException):
            current_session.alert.text

        recursive_compare(
            {
                "accepted": accept,
                "context": new_tab["context"],
                "type": "beforeunload",
            },
            closed_event,
        )

        if accept:
            assert current_session.url == page_target
        else:
            assert current_session.url == page_beforeunload

    return check_beforeunload_not_implicitly_accepted


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("accept", [False, True])
async def test_accept(check_beforeunload_not_implicitly_accepted, accept):
    await check_beforeunload_not_implicitly_accepted(accept)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("accept", [False, True])
async def test_accept_and_notify(check_beforeunload_not_implicitly_accepted, accept):
    await check_beforeunload_not_implicitly_accepted(accept)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("accept", [False, True])
async def test_dismiss(check_beforeunload_not_implicitly_accepted, accept):
    await check_beforeunload_not_implicitly_accepted(accept)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("accept", [False, True])
async def test_dismiss_and_notify(check_beforeunload_not_implicitly_accepted, accept):
    await check_beforeunload_not_implicitly_accepted(accept)


@pytest.mark.capabilities({"unhandledPromptBehavior": "ignore"})
@pytest.mark.parametrize("accept", [False, True])
async def test_ignore(check_beforeunload_not_implicitly_accepted, accept):
    await check_beforeunload_not_implicitly_accepted(accept)
