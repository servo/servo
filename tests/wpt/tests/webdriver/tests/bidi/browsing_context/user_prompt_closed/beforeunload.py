import asyncio
import pytest

import webdriver.bidi.error as error


pytestmark = pytest.mark.asyncio

USER_PROMPT_CLOSED_EVENT = "browsingContext.userPromptClosed"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest.mark.capabilities({"unhandledPromptBehavior": {'beforeUnload': 'ignore'}})
@pytest.mark.parametrize("accept", [False, True])
async def test_beforeunload(
    bidi_session,
    url,
    new_tab,
    subscribe_events,
    setup_beforeunload_page,
    wait_for_event,
    wait_for_future_safe,
    accept,
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])
    await setup_beforeunload_page(new_tab)

    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    navigated_future = asyncio.create_task(
        bidi_session.browsing_context.navigate(
            context=new_tab["context"],
            url=url("/webdriver/tests/support/html/default.html"),
            wait="none"
        )
    )

    # Wait for prompt to appear.
    await wait_for_future_safe(on_entry)

    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    # Just make sure the prompt is closed.
    event = await wait_for_future_safe(on_prompt_closed)
    assert event == {
        "context": new_tab["context"],
        "accepted": accept,
        "type": "beforeunload",
    }

    # Wait for the navigation to finish or fail.
    if accept:
        await navigated_future
    else:
        with pytest.raises(error.UnknownErrorException):
            await wait_for_future_safe(navigated_future)
