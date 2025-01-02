# META: timeout=long

import pytest
from tests.support.sync import AsyncPoll

from webdriver.bidi.modules.script import RealmTarget
from webdriver.error import TimeoutException
from ... import any_string, recursive_compare
from .. import create_sandbox


pytestmark = pytest.mark.asyncio

REALM_CREATED_EVENT = "script.realmCreated"


async def test_unsubscribe(bidi_session):
    await bidi_session.session.subscribe(events=[REALM_CREATED_EVENT])
    await bidi_session.session.unsubscribe(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.create(type_hint="tab")

    assert len(events) == 0

    remove_listener()


@pytest.mark.parametrize("type_hint", ["window", "tab"])
async def test_create_context(bidi_session, subscribe_events, type_hint):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    wait = AsyncPoll(
        bidi_session, message="Didn't receive realm created events")
    await wait.until(lambda _: len(events) >= 1)

    result = await bidi_session.script.get_realms(context=new_context["context"])

    assert events[-1] == result[0]

    remove_listener()


async def test_navigate(bidi_session, subscribe_events, inline, new_tab):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    result = await bidi_session.script.get_realms(context=new_tab["context"])

    assert events[-1] == result[0]

    remove_listener()


async def test_reload(bidi_session, subscribe_events, new_tab, inline):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.reload(
        context=new_tab["context"], wait="complete"
    )

    result = await bidi_session.script.get_realms(context=new_tab["context"])

    assert events[-1] == result[0]

    remove_listener()


@pytest.mark.parametrize("method", ["evaluate", "call_function"])
async def test_sandbox(
    bidi_session, subscribe_events, new_tab, wait_for_event, wait_for_future_safe, test_origin, method
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_origin, wait="complete"
    )
    await subscribe_events(events=[REALM_CREATED_EVENT])

    on_realm_created = wait_for_event(REALM_CREATED_EVENT)

    sandbox_name = "Test"
    sandbox_realm = await create_sandbox(
        bidi_session, new_tab["context"], sandbox_name, method
    )

    event = await wait_for_future_safe(on_realm_created)

    assert event == {
        "context": new_tab["context"],
        "origin": test_origin,
        "realm": sandbox_realm,
        "sandbox": sandbox_name,
        "type": "window",
    }


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(bidi_session, subscribe_events, top_context, inline, domain):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    frame_url = inline("<div>foo</div>")
    url = inline(f"<iframe src='{frame_url}'></iframe>", domain=domain)
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    realms = await bidi_session.script.get_realms()

    for realm in realms:
        # Find the relevant event for the specific realm
        event = [item for item in events if item["realm"] == realm["realm"]]
        assert event[0] == realm

    remove_listener()


async def test_subscribe_to_one_context(
    bidi_session, subscribe_events, new_tab, inline, top_context
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    # Subscribe to a specific context
    await subscribe_events(events=[REALM_CREATED_EVENT], contexts=[new_tab["context"]])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    # Make sure we didn't receive the event for the top context
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    result = await bidi_session.script.get_realms(context=new_tab["context"])

    assert events[-1] == result[0]

    remove_listener()


@pytest.mark.parametrize("method", ["evaluate", "call_function"])
async def test_script_when_realm_is_created(
    bidi_session, subscribe_events, new_tab, wait_for_event, wait_for_future_safe, inline, method
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    await subscribe_events(events=[REALM_CREATED_EVENT])

    on_realm_created = wait_for_event(REALM_CREATED_EVENT)

    await bidi_session.browsing_context.reload(context=new_tab["context"])

    realm_info = await wait_for_future_safe(on_realm_created)

    # Validate that it's possible to execute the script
    # as soon as a realm is created.
    if method == "evaluate":
        result = await bidi_session.script.evaluate(
            expression="1 + 2",
            await_promise=False,
            target=RealmTarget(realm_info["realm"]),
        )
    else:
        result = await bidi_session.script.call_function(
            function_declaration="() => 1 + 2",
            await_promise=False,
            target=RealmTarget(realm_info["realm"]),
        )

    assert result == {"type": "number", "value": 3}


async def test_dedicated_worker(
    wait_for_future_safe,
    bidi_session,
    subscribe_events,
    top_context,
    inline,
    event_loop,
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    window_realm = None
    worker_realm = event_loop.create_future()

    async def on_event(method, data):
        if data["type"] == "dedicated-worker":
            if worker_realm.done():
                raise "More than one dedicated worker"
            else:
                worker_realm.set_result(data)
        elif data["type"] == "window":
            nonlocal window_realm
            window_realm = data

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    worker_url = inline("while(true){}", doctype="js")
    url = inline(
        f"<script>const worker = new Worker('{worker_url}');</script>")
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    realm = await wait_for_future_safe(worker_realm)
    remove_listener()

    recursive_compare(
        {
            "type": "dedicated-worker",
            "realm": any_string,
            "origin": worker_url,
            "owners": [window_realm["realm"]],
        },
        realm,
    )


async def test_shared_worker(
    wait_for_future_safe,
    bidi_session,
    subscribe_events,
    top_context,
    inline,
    event_loop,
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    window_realm = None
    worker_realm = event_loop.create_future()

    async def on_event(method, data):
        if data["type"] == "shared-worker":
            if worker_realm.done():
                raise "More than one shared worker"
            else:
                worker_realm.set_result(data)
        elif data["type"] == "window":
            nonlocal window_realm
            window_realm = data

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    worker_url = inline("while(true){}", doctype="js")
    url = inline(
        f"""<script>
        const worker = new SharedWorker('{worker_url}');
    </script>"""
    )
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    realm = await wait_for_future_safe(worker_realm)
    remove_listener()

    recursive_compare(
        {
            "type": "shared-worker",
            "realm": any_string,
            "origin": worker_url,
        },
        realm,
    )


async def test_service_worker(
    wait_for_future_safe,
    bidi_session,
    subscribe_events,
    top_context,
    inline,
    event_loop,
):
    await subscribe_events(events=[REALM_CREATED_EVENT])

    window_realm = None
    worker_realm = event_loop.create_future()

    async def on_event(method, data):
        if data["type"] == "service-worker":
            if worker_realm.done():
                raise "More than one service worker"
            else:
                worker_realm.set_result(data)
        elif data["type"] == "window":
            nonlocal window_realm
            window_realm = data

    remove_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_event)

    worker_url = inline("while(true){}", doctype="js")
    url = inline(
        f"""<script>
        navigator.serviceWorker.register('{worker_url}');
        navigator.serviceWorker.startMessages();
    </script>"""
    )
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    realm = await wait_for_future_safe(worker_realm)
    remove_listener()

    recursive_compare(
        {
            "type": "service-worker",
            "realm": any_string,
            "origin": worker_url,
        },
        realm,
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_existing_realm(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, test_origin, type_hint):
    # See https://w3c.github.io/webdriver-bidi/#event-script-realmCreated
    # "The remote end subscribe steps with subscribe priority 2"
    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
        context=top_level_context["context"], url=test_origin, wait="complete"
    )

    on_entry = wait_for_event(REALM_CREATED_EVENT)
    await subscribe_events([REALM_CREATED_EVENT], contexts=[top_level_context["context"]])
    realm = await wait_for_future_safe(on_entry)

    recursive_compare(
        {
            "type": "window",
            "context": top_level_context["context"],
            "realm": any_string,
            "origin": test_origin,
        },
        realm,
    )
