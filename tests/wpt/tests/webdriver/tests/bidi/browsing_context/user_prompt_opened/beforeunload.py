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

    await bidi_session.send_command(
        "browsingContext.navigate",
        {
            "context": new_tab["context"],
            "url": url("/webdriver/tests/support/html/default.html"),
        },
    )

    event = await wait_for_future_safe(on_entry)

    recursive_compare(
        {
            "context": new_tab["context"],
            "type": "beforeunload",
            "message": any_string,
        },
        event,
    )
