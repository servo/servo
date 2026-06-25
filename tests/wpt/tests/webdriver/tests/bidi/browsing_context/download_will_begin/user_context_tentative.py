import random

import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

DOWNLOAD_WILL_BEGIN = "browsingContext.downloadWillBegin"


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    subscribe_events,
    create_user_context,
    inline,
    wait_for_event,
    wait_for_future_safe,
    expect_download_end,
    user_context,
):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )

    filename = f"download_{random.random()}.txt"
    download_link = "data:text/plain;charset=utf-8,test_content"
    page_url = inline(
        f"""<a id="d" href="{download_link}" download="{filename}">download</a>"""
    )
    await bidi_session.browsing_context.navigate(
        context=context["context"], url=page_url, wait="complete"
    )
    await subscribe_events(events=[DOWNLOAD_WILL_BEGIN])
    on_entry = wait_for_event(DOWNLOAD_WILL_BEGIN)
    expect_download_end(1)
    await bidi_session.script.evaluate(
        expression="document.getElementById('d').click()",
        target=ContextTarget(context["context"]),
        await_promise=True,
        user_activation=True,
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
