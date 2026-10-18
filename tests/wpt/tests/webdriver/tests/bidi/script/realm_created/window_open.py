import pytest
from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll


pytestmark = pytest.mark.asyncio

CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"
REALM_CREATED_EVENT = "script.realmCreated"


@pytest.mark.parametrize("window_url", ["", "about:blank", "inline"])
async def test_window_open(
    bidi_session, configuration, subscribe_events, top_context, inline, window_url
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)

    if window_url == "inline":
        window_url = inline("<div>in window</div>")

    await bidi_session.script.evaluate(
        expression=f"window.open('{window_url}')",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    # The initial about:blank document of the new window can get its own realm
    # before it is replaced, so wait until the last event matches the realm of
    # the window's active document.
    async def check_window_realm(_):
        realms = await bidi_session.script.get_realms()
        window_realm = None
        for realm in realms:
            if realm["context"] != top_context["context"]:
                window_realm = realm

        assert len(events) > 0
        assert events[-1] == window_realm

    wait = AsyncPoll(bidi_session, timeout=2 * configuration["timeout_multiplier"])
    await wait.until(check_window_realm)

    remove_listener()


@pytest.mark.parametrize("window_url", ["", "about:blank", "inline"])
async def test_event_order(
    bidi_session, subscribe_events, wait_for_bidi_events, new_tab, inline, window_url
):
    await subscribe_events(events=[CONTEXT_CREATED_EVENT, REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(method)

    remove_listener_for_context_created = bidi_session.add_event_listener(
        CONTEXT_CREATED_EVENT, on_event
    )
    remove_listener_for_realm_created = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event
    )

    if window_url == "inline":
        window_url = inline("<div>in window</div>")

    events = []

    await bidi_session.script.evaluate(
        expression=f"window.open('{window_url}')",
        await_promise=False,
        target=ContextTarget(new_tab["context"]),
    )

    # A realm for the initial about:blank document might be reported as well.
    await wait_for_bidi_events(events, 2, equal_check=False)

    assert events[0] == CONTEXT_CREATED_EVENT
    assert events[1] == REALM_CREATED_EVENT

    remove_listener_for_context_created()
    remove_listener_for_realm_created()
