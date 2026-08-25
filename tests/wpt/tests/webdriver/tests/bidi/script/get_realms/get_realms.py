import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio

REALM_CREATED_EVENT = "script.realmCreated"


PAGE_ABOUT_BLANK = "about:blank"


async def test_payload_types(bidi_session):
    result = await bidi_session.script.get_realms()

    recursive_compare(
        [
            {
                "origin": any_string,
                "realm": any_string,
                "type": any_string,
            }
        ],
        result,
    )


async def test_realm_is_consistent_when_calling_twice(bidi_session):
    result = await bidi_session.script.get_realms()

    result_calling_again = await bidi_session.script.get_realms()

    assert result[0]["realm"] == result_calling_again[0]["realm"]


async def test_realm_is_different_after_reload(bidi_session, top_context):
    result = await bidi_session.script.get_realms()

    # Reload the page
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=PAGE_ABOUT_BLANK, wait="complete"
    )

    result_after_reload = await bidi_session.script.get_realms()

    assert result[0]["realm"] != result_after_reload[0]["realm"]


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_multiple_top_level_contexts(bidi_session, top_context, type_hint):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    result = await bidi_session.script.get_realms()

    # Evaluate to get realm ids
    top_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    new_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(new_context["context"]),
        await_promise=False,
    )

    recursive_compare(
        [
            {
                "context": top_context["context"],
                "origin": "null",
                "realm": top_context_result["realm"],
                "type": "window",
            },
            {
                "context": new_context["context"],
                "origin": "null",
                "realm": new_context_result["realm"],
                "type": "window",
            },
        ],
        result,
    )


async def test_iframes(
    bidi_session,
    top_context,
    test_alt_origin,
    test_origin,
    test_page_cross_origin_frame,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_page_cross_origin_frame,
        wait="complete",
    )

    result = await bidi_session.script.get_realms()

    # Evaluate to get realm id
    top_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1
    frame_context = frames[0]["context"]

    # Evaluate to get realm id
    frame_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(frame_context),
        await_promise=False,
    )

    recursive_compare(
        [
            {
                "context": top_context["context"],
                "origin": test_origin,
                "realm": top_context_result["realm"],
                "type": "window",
            },
            {
                "context": frame_context,
                "origin": test_alt_origin,
                "realm": frame_context_result["realm"],
                "type": "window",
            },
        ],
        result,
    )

    # Clean up origin
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=PAGE_ABOUT_BLANK, wait="complete"
    )


async def test_origin(bidi_session, inline, top_context, test_origin):
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.script.get_realms()

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
                "origin": test_origin,
                "realm": top_context_result["realm"],
                "type": "window",
            }
        ],
        result,
    )

    # Clean up origin
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=PAGE_ABOUT_BLANK, wait="complete"
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

    result = await bidi_session.script.get_realms()

    # Check window realms are returned and retrieve them
    assert any(r["type"] == "window" for r in result)
    window_realms = [r for r in result if r["type"] == "window"]
    assert len(window_realms) == 1

    assert any(r["type"] == "dedicated-worker" for r in result)
    dedicated_worker_realms = [r for r in result if r["type"] == "dedicated-worker"]
    assert len(dedicated_worker_realms) == 1

    recursive_compare(
        [
            {
                "type": "dedicated-worker",
                "realm": any_string,
                "origin": worker_url,
                "owners": [window_realms[0]["realm"]],
            }
        ],
        dedicated_worker_realms,
    )


async def test_dedicated_worker_with_context(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_bidi_events,
    inline,
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    await subscribe_events(events=[REALM_CREATED_EVENT])

    worker_events = []

    async def on_event(method, data):
        if data["type"] == "dedicated-worker":
            worker_events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)

    worker_url = inline("console.log('dedicated worker')", doctype="js")
    url = inline(f"<script>const worker = new Worker('{worker_url}');</script>")
    await bidi_session.browsing_context.navigate(
        url=url, context=top_context["context"], wait="complete"
    )

    await wait_for_bidi_events(worker_events, 1)
    remove_listener()

    result = await bidi_session.script.get_realms(context=top_context["context"])

    assert any(r["type"] == "dedicated-worker" for r in result)

    result_new_context = await bidi_session.script.get_realms(
        context=new_context["context"]
    )

    assert not any(r["type"] == "dedicated-worker" for r in result_new_context)
