import pytest

from . import assert_javascript_entry, create_log
from ... import int_interval


@pytest.mark.asyncio
async def test_types_and_values(
    bidi_session, subscribe_events, current_time, top_context, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    time_start = await current_time()

    expected_text = await create_log(bidi_session, top_context, "javascript_error", "cached_message")
    event_data = await wait_for_future_safe(on_entry_added)

    time_end = await current_time()

    assert_javascript_entry(
        event_data,
        level="error",
        text=expected_text,
        timestamp=int_interval(time_start, time_end),
        context=top_context["context"],
    )
