import asyncio

import pytest

from ... import get_viewport_dimensions


pytestmark = pytest.mark.asyncio


async def test_when_navigation_started(
    bidi_session, inline, url, new_tab, wait_for_event, wait_for_future_safe
):
    test_viewport = {"width": 499, "height": 599}

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>Foo"), wait="complete"
    )

    await bidi_session.session.subscribe(events=["browsingContext.navigationStarted"])
    on_navigation_started = wait_for_event("browsingContext.navigationStarted")

    # Save the navigation task and await for it later.
    slow_page = url("/webdriver/tests/support/html/default.html?pipe=trickle(d1)")
    navigation_future = asyncio.create_task(
        bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=slow_page, wait="complete"
        )
    )

    await wait_for_future_safe(on_navigation_started)
    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"], viewport=test_viewport
    )

    await navigation_future

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport
