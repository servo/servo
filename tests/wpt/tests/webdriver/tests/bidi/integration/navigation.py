import pytest

from webdriver.error import TimeoutException
from webdriver.bidi.error import UnknownErrorException
from webdriver.bidi.modules.script import ContextTarget


pytestmark = pytest.mark.asyncio

NAVIGATION_STARTED_EVENT = "browsingContext.navigationStarted"
FRAGMENT_NAVIGATED_EVENT = "browsingContext.fragmentNavigated"
BEFORE_REQUEST_SENT_EVENT = "network.beforeRequestSent"

async def test_navigate_history_replacestate_beforeunload(
    bidi_session, inline, new_tab, subscribe_events
):
    url = inline("""
        <script>
          window.addEventListener(
            'beforeunload',
            () => {
              return history.replaceState(null, 'initial', window.location.href);
            },
            false
          );
        </script>""")

    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    events = []

    async def on_event(method, data):
        events.append(method)

    remove_navigation_started_listener = bidi_session.add_event_listener(
        NAVIGATION_STARTED_EVENT, on_event
    )

    remove_fragment_navigated_listener = bidi_session.add_event_listener(
        FRAGMENT_NAVIGATED_EVENT, on_event
    )

    remove_before_request_sent_listener = bidi_session.add_event_listener(
        BEFORE_REQUEST_SENT_EVENT, on_event
    )

    await subscribe_events([NAVIGATION_STARTED_EVENT, FRAGMENT_NAVIGATED_EVENT, BEFORE_REQUEST_SENT_EVENT])

    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="interactive"
    )

    # Navigation caused by browsing_context.navigate call should happen first:
    # https://html.spec.whatwg.org/#beginning-navigation
    # network.beforeRequestSent should arrive before the page becomes
    # interactive.
    assert events == [
        NAVIGATION_STARTED_EVENT,
        FRAGMENT_NAVIGATED_EVENT,
        BEFORE_REQUEST_SENT_EVENT
    ]

    remove_navigation_started_listener()
    remove_fragment_navigated_listener()
    remove_before_request_sent_listener()


async def test_navigate_started_and_before_request_sent_event_order(
    bidi_session, new_tab, inline, subscribe_events
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<div>foo</div>"),
        wait="complete",
    )

    await subscribe_events([NAVIGATION_STARTED_EVENT, BEFORE_REQUEST_SENT_EVENT])

    events = []

    async def on_event(method, data):
        events.append(method)

    remove_navigation_started_listener = bidi_session.add_event_listener(
        NAVIGATION_STARTED_EVENT, on_event
    )

    remove_before_request_sent_listener = bidi_session.add_event_listener(
        BEFORE_REQUEST_SENT_EVENT, on_event
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>bar</div>"), wait="complete"
    )

    # Navigation caused by browsing_context.navigate call should happen first:
    # https://html.spec.whatwg.org/#beginning-navigation
    # network.beforeRequestSent should arrive before the page becomes
    # interactive.
    assert events == [NAVIGATION_STARTED_EVENT, BEFORE_REQUEST_SENT_EVENT]

    remove_navigation_started_listener()
    remove_before_request_sent_listener()
