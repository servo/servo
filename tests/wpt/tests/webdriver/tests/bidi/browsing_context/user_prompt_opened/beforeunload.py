import asyncio
import pytest

from .. import (
    any_string,
    recursive_compare,
)


pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


async def test_beforeunload(
    bidi_session,
    subscribe_events,
    url,
    new_tab,
    setup_beforeunload_page,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await setup_beforeunload_page(new_tab)

    navigation_future = asyncio.create_task(
        bidi_session.browsing_context.navigate(
            context=new_tab["context"],
            url=url("/webdriver/tests/support/html/default.html"),
            wait="none"
        )
    )

    event = await wait_for_future_safe(on_entry)

    recursive_compare(
        {
            "context": new_tab["context"],
            "type": "beforeunload",
            "message": any_string,
            **({"userContext": new_tab["userContext"]} if "userContext" in event else {})
        },
        event,
    )

    # The beforeunload prompt is accepted by default, so the navigation is
    # expected to proceed. Wait for the navigation command to settle so that
    # no command is still pending when the tab gets closed during teardown.
    await wait_for_future_safe(navigation_future)
