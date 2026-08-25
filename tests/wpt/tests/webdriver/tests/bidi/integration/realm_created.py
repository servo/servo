import pytest

pytestmark = pytest.mark.asyncio

BROWSING_CONTEXT_CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"
REALM_CREATED_EVENT = "script.realmCreated"


async def test_with_context_created(
    bidi_session,
    subscribe_events,
    new_tab,
    inline
):
    page_html = "<div>test</div>"

    await bidi_session.browsing_context.navigate(
        url=inline(page_html, domain=""),
        context=new_tab["context"],
        wait="complete"
    )

    # For Firefox we want to subscribe to "browsingContext.contextCreated"
    # to verify that the "script.realmCreated" event is still sent, since
    # in this case it will have to wait for
    # the "browsingContext.contextCreated" event to be sent first.
    # See https://bugzilla.mozilla.org/show_bug.cgi?id=2042385
    # for more information.
    await subscribe_events([
        REALM_CREATED_EVENT,
        BROWSING_CONTEXT_CONTEXT_CREATED_EVENT,
    ])

    # Track all received "script.realmCreated" events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        url=inline(page_html, domain="alt"), context=new_tab["context"], wait="complete"
    )

    result = await bidi_session.script.get_realms(context=new_tab["context"])

    assert events[-1] == result[0]

    remove_listener()
