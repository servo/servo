import asyncio

import pytest

pytestmark = pytest.mark.asyncio

from .. import AUTH_REQUIRED_EVENT, PAGE_EMPTY_HTML


# This test can be moved back to `auth_required.py` when all implementations
# support handing of HTTP auth prompt.
async def test_unsubscribe(bidi_session, new_tab, url):
    await bidi_session.session.subscribe(events=[AUTH_REQUIRED_EVENT])
    await bidi_session.session.unsubscribe(events=[AUTH_REQUIRED_EVENT])

    # Track all received network.authRequired events in the events array.
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(AUTH_REQUIRED_EVENT, on_event)

    # Navigate to authentication.py again and check no event is received.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(
            "/webdriver/tests/support/http_handlers/authentication.py?realm=testrealm"
        ),
        wait="none",
    )
    await asyncio.sleep(0.5)
    assert len(events) == 0

    remove_listener()
