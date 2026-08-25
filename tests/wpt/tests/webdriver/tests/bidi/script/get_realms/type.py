import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio


PAGE_ABOUT_BLANK = "about:blank"
REALM_CREATED_EVENT = "script.realmCreated"


async def test_type(bidi_session, top_context):
    result = await bidi_session.script.get_realms(type="window")

    # Evaluate to get realm id
    top_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(
        [
            {
                "context": top_context["context"],
                "origin": "null",
                "realm": top_context_result["realm"],
                "type": "window",
            }
        ],
        result,
    )


async def test_dedicated_worker(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_bidi_events,
    inline,
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        if data["type"] == "dedicated-worker":
            events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)

    worker_url = inline("console.log('dedicated worker')", doctype="js")
    url = inline(f"<script>const worker = new Worker('{worker_url}');</script>")
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    await wait_for_bidi_events(events, 1)
    remove_listener()

    result = await bidi_session.script.get_realms(type="dedicated-worker")

    window_realms = await bidi_session.script.get_realms(
        context=top_context["context"], type="window"
    )

    recursive_compare(
        [
            {
                "type": "dedicated-worker",
                "realm": any_string,
                "origin": worker_url,
                "owners": [window_realms[0]["realm"]],
            }
        ],
        result,
    )


async def test_shared_worker(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_bidi_events,
    inline,
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        if data["type"] == "shared-worker":
            events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)

    worker_url = inline("console.log('shared worker')", doctype="js")
    url = inline(
        f"""<script>
        const worker = new SharedWorker('{worker_url}');
    </script>"""
    )
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    await wait_for_bidi_events(events, 1)
    remove_listener()

    result = await bidi_session.script.get_realms(type="shared-worker")

    recursive_compare(
        [
            {
                "type": "shared-worker",
                "realm": any_string,
                "origin": worker_url,
            }
        ],
        result,
    )


async def test_service_worker(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_bidi_events,
    inline,
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        if data["type"] == "service-worker":
            events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)

    worker_url = inline("console.log('service worker')", doctype="js")
    url = inline(
        f"""<script>
        window.onRegistration =
          navigator.serviceWorker.register('{worker_url}');
        navigator.serviceWorker.startMessages();
    </script>"""
    )
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    await wait_for_bidi_events(events, 1)
    remove_listener()

    result = await bidi_session.script.get_realms(type="service-worker")

    recursive_compare(
        [
            {
                "type": "service-worker",
                "realm": any_string,
                "origin": worker_url,
            }
        ],
        result,
    )

    # Unregister the service worker registration.
    await bidi_session.script.evaluate(
        expression="""window.onRegistration.then(r => r.unregister())""",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )
