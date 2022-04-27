import math
import time

import pytest

from . import assert_javascript_entry


@pytest.mark.asyncio
async def test_types_and_values(
    bidi_session, current_session, current_time, inline, top_context, wait_for_event
):
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    expected_text = current_session.execute_script(
        "const err = new Error('foo'); return err.toString()")

    time_start = current_time()

    url = inline("<script>function bar() { throw new Error('foo'); }; bar();</script>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    event_data = await on_entry_added

    time_end = current_time()

    assert_javascript_entry(
        event_data,
        level="error",
        text=expected_text,
        time_start=time_start,
        time_end=time_end
    )

    # Navigate to a page with no error to avoid polluting the next tests with
    # JavaScript errors.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline("<p>foo"),
        wait="complete",
    )
