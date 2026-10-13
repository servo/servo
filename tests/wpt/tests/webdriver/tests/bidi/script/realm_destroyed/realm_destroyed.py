# META: timeout=long

# Longer timeout required due to a large number of script evaluation and serialization subtests.

import asyncio
import pytest
from webdriver.bidi.error import BidiException
from webdriver.bidi.modules.script import ContextTarget, RealmTarget
from webdriver.error import TimeoutException

from ..realm_created.realm_created import REALM_CREATED_EVENT
from .. import create_sandbox


pytestmark = pytest.mark.asyncio

REALM_DESTROYED_EVENT = "script.realmDestroyed"


async def test_unsubscribe(bidi_session):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.session.subscribe(events=[REALM_DESTROYED_EVENT])
    await bidi_session.session.unsubscribe(events=[REALM_DESTROYED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.close(context=new_context["context"])

    assert len(events) == 0

    remove_listener()


@pytest.mark.parametrize("type_hint", ["window", "tab"])
async def test_close_context(bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, type_hint):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    result = await bidi_session.script.get_realms(context=new_context["context"])

    on_realm_destroyed = wait_for_event(REALM_DESTROYED_EVENT)

    await bidi_session.browsing_context.close(context=new_context["context"])

    event = await wait_for_future_safe(on_realm_destroyed)

    assert event == {"realm": result[0]["realm"]}


async def test_navigate(
    bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, inline, new_tab
):
    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    result = await bidi_session.script.get_realms(context=new_tab["context"])

    on_realm_destroyed = wait_for_event(REALM_DESTROYED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    event = await wait_for_future_safe(on_realm_destroyed)

    assert event == {"realm": result[0]["realm"]}


async def test_reload_context(
    bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, top_context
):
    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    result = await bidi_session.script.get_realms(context=top_context["context"])

    on_realm_destroyed = wait_for_event(REALM_DESTROYED_EVENT)

    await bidi_session.browsing_context.reload(context=top_context["context"])

    event = await wait_for_future_safe(on_realm_destroyed)

    assert event == {"realm": result[0]["realm"]}


@pytest.mark.parametrize("method", ["evaluate", "call_function"])
async def test_sandbox(bidi_session, subscribe_events, wait_for_bidi_events, new_tab, method):
    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    # Track all received script.realmDestroyed events in the destroyed_realm_ids array
    destroyed_realm_ids = []

    async def on_event(method, data):
        destroyed_realm_ids.append(data["realm"])

    remove_listener = bidi_session.add_event_listener(REALM_DESTROYED_EVENT, on_event)

    sandbox_realm = await create_sandbox(
        bidi_session, new_tab["context"], "test", method
    )

    await bidi_session.browsing_context.close(context=new_tab["context"])

    await wait_for_bidi_events(destroyed_realm_ids, 2)

    assert sandbox_realm in destroyed_realm_ids

    remove_listener()


async def test_subscribe_after_sandbox_creation(
    bidi_session, subscribe_events, wait_for_bidi_events, new_tab, inline
):
    sandbox_realm = await create_sandbox(bidi_session, new_tab["context"])

    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    # Track all received script.realmDestroyed events in the destroyed_realm_ids array
    destroyed_realm_ids = []

    async def on_event(method, data):
        if data["realm"] == sandbox_realm:
            destroyed_realm_ids.append(data["realm"])

    remove_listener = bidi_session.add_event_listener(REALM_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    await wait_for_bidi_events(destroyed_realm_ids, 1)

    remove_listener()


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session, subscribe_events, top_context, inline, wait_for_event, wait_for_future_safe, domain, iframe
):
    frame_html = "<div>foo</div>"
    url = inline(iframe(frame_html), domain=domain)
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    on_realm_destroyed = wait_for_event(REALM_DESTROYED_EVENT)

    frame_context = contexts[0]["children"][0]["context"]
    result = await bidi_session.script.get_realms(context=frame_context)

    await bidi_session.browsing_context.navigate(
        context=frame_context, url=inline("<div>foo</div>"), wait="complete"
    )

    event = await wait_for_future_safe(on_realm_destroyed)

    assert event == {"realm": result[0]["realm"]}


async def test_iframe_destroy_parent(
    bidi_session, subscribe_events, test_page_same_origin_frame, wait_for_bidi_events, new_tab
):
    await bidi_session.browsing_context.navigate(
        url=test_page_same_origin_frame, context=new_tab["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    await subscribe_events(events=[REALM_DESTROYED_EVENT])

    # Track all received script.realmDestroyed events in the destroyed_realm_ids array
    destroyed_realm_ids = []

    async def on_event(method, data):
        destroyed_realm_ids.append(data["realm"])

    remove_listener = bidi_session.add_event_listener(REALM_DESTROYED_EVENT, on_event)

    realm_for_iframe = await bidi_session.script.get_realms(
        context=contexts[0]["children"][0]["context"]
    )
    realm_for_parent = await bidi_session.script.get_realms(context=new_tab["context"])

    await bidi_session.browsing_context.close(context=new_tab["context"])

    await wait_for_bidi_events(destroyed_realm_ids, 2)

    assert realm_for_iframe[0]["realm"] in destroyed_realm_ids
    assert realm_for_parent[0]["realm"] in destroyed_realm_ids

    remove_listener()


async def test_subscribe_to_one_context(
    bidi_session, subscribe_events, wait_for_bidi_events, new_tab, inline, top_context
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    # Subscribe to a specific context
    await subscribe_events(
        events=[REALM_DESTROYED_EVENT], contexts=[new_tab["context"]]
    )

    # Track all received script.realmDestroyed events in the destroyed_realm_ids array
    destroyed_realm_ids = []

    async def on_event(method, data):
        destroyed_realm_ids.append(data["realm"])

    remove_listener = bidi_session.add_event_listener(REALM_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    # Make sure we didn't receive the event for the top context
    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(destroyed_realm_ids, 1, timeout=0.5)

    result = await bidi_session.script.get_realms(context=new_tab["context"])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    await wait_for_bidi_events(destroyed_realm_ids, 1)

    assert result[0]["realm"] in destroyed_realm_ids

    remove_listener()


async def test_dedicated_worker(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_bidi_events,
    inline,
):
    await subscribe_events(events=[REALM_CREATED_EVENT, REALM_DESTROYED_EVENT])

    created_events = []
    destroyed_events = []

    async def on_realm_created_event(method, data):
        if data["type"] == "dedicated-worker":
            created_events.append(data)

    async def on_realm_destroyed_event(method, data):
        if len(created_events) > 0 and data["realm"] == created_events[0]["realm"]:
            destroyed_events.append(data)

    remove_realm_created_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_realm_created_event
    )
    remove_realm_destroyed_listener = bidi_session.add_event_listener(
        REALM_DESTROYED_EVENT, on_realm_destroyed_event
    )

    worker_url = inline("setInterval(()=>{}, 1)", doctype="js")
    url = inline(f"<script>window.worker = new Worker('{worker_url}');</script>")
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    await wait_for_bidi_events(created_events, 1)

    # Terminate the worker only after its realm is created to avoid racing with
    # worker initialization.
    await bidi_session.script.evaluate(
        expression="window.worker.terminate()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    await wait_for_bidi_events(destroyed_events, 1)

    assert len(created_events) == 1
    assert len(destroyed_events) == 1
    assert destroyed_events[0]["realm"] == created_events[0]["realm"]

    remove_realm_created_listener()
    remove_realm_destroyed_listener()


async def test_shared_worker(
    bidi_session,
    subscribe_events,
    new_tab,
    wait_for_bidi_events,
    inline,
):
    await subscribe_events(events=[REALM_CREATED_EVENT, REALM_DESTROYED_EVENT])

    created_events = []
    destroyed_events = []

    async def on_realm_created_event(method, data):
        if data["type"] == "shared-worker":
            created_events.append(data)

    async def on_realm_destroyed_event(method, data):
        if len(created_events) > 0 and data["realm"] == created_events[0]["realm"]:
            destroyed_events.append(data)

    remove_realm_created_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_realm_created_event
    )
    remove_realm_destroyed_listener = bidi_session.add_event_listener(
        REALM_DESTROYED_EVENT, on_realm_destroyed_event
    )

    worker_url = inline("console.log('shared worker')", doctype="js")
    url = inline(
        f"""<script>
        const worker = new SharedWorker('{worker_url}');
    </script>"""
    )
    # Create the shared worker in `new_tab` instead of `top_context` so the tab
    # can be closed to destroy the worker.
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    await wait_for_bidi_events(created_events, 1)

    # Close the tab instead of navigating away so BFCache cannot keep the
    # document and its shared worker alive.
    await bidi_session.browsing_context.close(context=new_tab["context"])

    await wait_for_bidi_events(destroyed_events, 1)

    assert len(created_events) == 1
    assert len(destroyed_events) == 1
    assert destroyed_events[0]["realm"] == created_events[0]["realm"]

    remove_realm_created_listener()
    remove_realm_destroyed_listener()


async def test_dedicated_worker_subscribe_to_one_context(
    bidi_session,
    subscribe_events,
    wait_for_bidi_events,
    new_tab,
    top_context,
    inline,
):
    # Verify that subscribing to `script.realmDestroyed` for a single context
    # (`new_tab`) does not emit events when a worker in an unsubscribed context
    # (`top_context`) is destroyed.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>bar</div>"), wait="complete"
    )

    # Subscribe to `script.realmCreated` globally so we can wait for workers in
    # both the unsubscribed and subscribed contexts to initialize before
    # terminating them.
    await subscribe_events(events=[REALM_CREATED_EVENT])
    await subscribe_events(
        events=[REALM_DESTROYED_EVENT],
        contexts=[new_tab["context"]]
    )

    created_events = []
    destroyed_events = []

    async def on_realm_created_event(method, data):
        if data["type"] == "dedicated-worker":
            created_events.append(data)

    # Record destruction events for any created worker so an unexpected event
    # from the unsubscribed context is caught.
    async def on_realm_destroyed_event(method, data):
        if data["realm"] in [e["realm"] for e in created_events]:
            destroyed_events.append(data)

    remove_realm_created_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_realm_created_event
    )
    remove_realm_destroyed_listener = bidi_session.add_event_listener(
        REALM_DESTROYED_EVENT, on_realm_destroyed_event
    )

    # First, create and terminate a worker in the unsubscribed context.
    worker_url_unsubscribed = inline("setInterval(()=>{}, 1)", doctype="js")
    url_unsubscribed = inline(
        f"<script>window.worker = new Worker('{worker_url_unsubscribed}');</script>"
    )
    await bidi_session.browsing_context.navigate(
        url=url_unsubscribed, context=top_context["context"], wait="complete"
    )
    await wait_for_bidi_events(created_events, 1)

    # Start a pending evaluation in the unsubscribed worker realm that will
    # reject as soon as the worker realm is destroyed.
    unsubscribed_eval_task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="new Promise(() => {})",
            target=RealmTarget(created_events[0]["realm"]),
            await_promise=True,
        )
    )

    # Terminate the worker only after its realm is created to avoid racing with
    # worker initialization.
    await bidi_session.script.evaluate(
        expression="window.worker.terminate()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    # Wait for the pending evaluation in the unsubscribed worker realm to fail,
    # guaranteeing that the remote end has processed its realm destruction.
    with pytest.raises(BidiException):
        await unsubscribed_eval_task

    # Next, create and terminate a worker in the subscribed context.
    worker_url = inline("setInterval(()=>{}, 1)", doctype="js")
    url = inline(f"<script>window.worker = new Worker('{worker_url}');</script>")
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    await wait_for_bidi_events(created_events, 2)

    await bidi_session.script.evaluate(
        expression="window.worker.terminate()",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    await wait_for_bidi_events(destroyed_events, 1)

    # Only the second (subscribed) worker's realm destruction should be reported.
    assert len(created_events) == 2
    assert len(destroyed_events) == 1
    assert destroyed_events[0]["realm"] == created_events[1]["realm"]

    remove_realm_created_listener()
    remove_realm_destroyed_listener()


async def test_dedicated_worker_subscribe_to_user_context(
    bidi_session,
    subscribe_events,
    wait_for_bidi_events,
    create_user_context,
    inline,
):
    # Verify that subscribing to `script.realmDestroyed` for `user_context_a`
    # does not emit events when a worker in the default user context is
    # destroyed.
    user_context_a = await create_user_context()
    context_a = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_a
    )

    await bidi_session.browsing_context.navigate(
        context=context_a["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    # Create a context in the unsubscribed default user context.
    context_b = await bidi_session.browsing_context.create(type_hint="tab")

    # Subscribe to `script.realmCreated` globally so we can wait for workers in
    # both user contexts to initialize before terminating them.
    await subscribe_events(events=[REALM_CREATED_EVENT])
    await subscribe_events(
        events=[REALM_DESTROYED_EVENT],
        user_contexts=[user_context_a]
    )

    created_events = []
    destroyed_events = []

    async def on_realm_created_event(method, data):
        if data["type"] == "dedicated-worker":
            created_events.append(data)

    # Record destruction events for any created worker so an unexpected event
    # from the unsubscribed user context is caught.
    async def on_realm_destroyed_event(method, data):
        if data["realm"] in [e["realm"] for e in created_events]:
            destroyed_events.append(data)

    remove_realm_created_listener = bidi_session.add_event_listener(
        REALM_CREATED_EVENT, on_realm_created_event
    )
    remove_realm_destroyed_listener = bidi_session.add_event_listener(
        REALM_DESTROYED_EVENT, on_realm_destroyed_event
    )

    # First, create and terminate a worker in the unsubscribed user context.
    worker_url_unsubscribed = inline("setInterval(()=>{}, 1)", doctype="js")
    url_unsubscribed = inline(
        f"<script>window.worker = new Worker('{worker_url_unsubscribed}');</script>"
    )
    await bidi_session.browsing_context.navigate(
        url=url_unsubscribed, context=context_b["context"], wait="complete"
    )
    await wait_for_bidi_events(created_events, 1)

    # Start a pending evaluation in the unsubscribed worker realm that will
    # reject as soon as the worker realm is destroyed.
    unsubscribed_eval_task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="new Promise(() => {})",
            target=RealmTarget(created_events[0]["realm"]),
            await_promise=True,
        )
    )

    # Terminate the worker only after its realm is created to avoid racing with
    # worker initialization.
    await bidi_session.script.evaluate(
        expression="window.worker.terminate()",
        target=ContextTarget(context_b["context"]),
        await_promise=False,
    )

    # Wait for the pending evaluation in the unsubscribed worker realm to fail,
    # guaranteeing that the remote end has processed its realm destruction.
    with pytest.raises(BidiException):
        await unsubscribed_eval_task

    # Next, create and terminate a worker in the subscribed user context.
    worker_url = inline("setInterval(()=>{}, 1)", doctype="js")
    url = inline(f"<script>window.worker = new Worker('{worker_url}');</script>")
    await bidi_session.browsing_context.navigate(
        url=url, context=context_a["context"], wait="complete"
    )

    await wait_for_bidi_events(created_events, 2)

    await bidi_session.script.evaluate(
        expression="window.worker.terminate()",
        target=ContextTarget(context_a["context"]),
        await_promise=False,
    )

    await wait_for_bidi_events(destroyed_events, 1)

    # Only the second (subscribed) worker's realm destruction should be reported.
    assert len(created_events) == 2
    assert len(destroyed_events) == 1
    assert destroyed_events[0]["realm"] == created_events[1]["realm"]

    remove_realm_created_listener()
    remove_realm_destroyed_listener()
