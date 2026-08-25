import pytest

import asyncio

from webdriver.bidi.modules.script import ScriptEvaluateResultException

from .. import AUTH_REQUIRED_EVENT, PAGE_EMPTY_HTML

pytestmark = pytest.mark.asyncio

# This test can be moved back to `auth_required.py` when all implementations
# support handing of HTTP auth prompt.
async def test_unsubscribe(bidi_session, new_tab, url, fetch):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    await bidi_session.session.subscribe(events=[AUTH_REQUIRED_EVENT])
    await bidi_session.session.unsubscribe(events=[AUTH_REQUIRED_EVENT])

    # Track all received network.authRequired events in the events array.
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        AUTH_REQUIRED_EVENT, on_event)

    asyncio.ensure_future(fetch(url=url(
        "/webdriver/tests/support/http_handlers/authentication.py?realm=testrealm"
    ), context=new_tab))

    assert len(events) == 0

    remove_listener()
