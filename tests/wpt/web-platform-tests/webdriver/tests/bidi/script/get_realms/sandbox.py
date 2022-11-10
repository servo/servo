import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare

PAGE_ABOUT_BLANK = "about:blank"


@pytest.mark.asyncio
async def test_sandbox(bidi_session, top_context):
    evaluate_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    # Create a sandbox
    evaluate_in_sandbox_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"], "sandbox"),
        await_promise=False,
    )

    result = await bidi_session.script.get_realms()

    recursive_compare(
        [
            {
                "context": top_context["context"],
                "origin": "null",
                "realm": evaluate_result["realm"],
                "type": "window",
            },
            {
                "context": top_context["context"],
                "origin": "null",
                "realm": evaluate_in_sandbox_result["realm"],
                "sandbox": "sandbox",
                "type": "window",
            },
        ],
        result,
    )

    # Reload to clean up sandboxes
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=PAGE_ABOUT_BLANK, wait="complete"
    )


@pytest.mark.asyncio
async def test_origin(bidi_session, inline, top_context, test_origin):
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    evaluate_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    # Create a sandbox
    evaluate_in_sandbox_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"], "sandbox"),
        await_promise=False,
    )

    result = await bidi_session.script.get_realms()

    recursive_compare(
        [
            {
                "context": top_context["context"],
                "origin": test_origin,
                "realm": evaluate_result["realm"],
                "type": "window",
            },
            {
                "context": top_context["context"],
                "origin": test_origin,
                "realm": evaluate_in_sandbox_result["realm"],
                "sandbox": "sandbox",
                "type": "window",
            },
        ],
        result,
    )

    # Reload to clean up sandboxes
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=PAGE_ABOUT_BLANK, wait="complete"
    )


@pytest.mark.asyncio
async def test_type(bidi_session, top_context):
    evaluate_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    # Create a sandbox
    evaluate_in_sandbox_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(top_context["context"], "sandbox"),
        await_promise=False,
    )

    # Should be extended when more types are supported
    result = await bidi_session.script.get_realms(type="window")

    recursive_compare(
        [
            {
                "context": top_context["context"],
                "origin": "null",
                "realm": evaluate_result["realm"],
                "type": "window",
            },
            {
                "context": top_context["context"],
                "origin": "null",
                "realm": evaluate_in_sandbox_result["realm"],
                "sandbox": "sandbox",
                "type": "window",
            },
        ],
        result,
    )

    # Reload to clean up sandboxes
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=PAGE_ABOUT_BLANK, wait="complete"
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_multiple_top_level_contexts(
    bidi_session,
    test_alt_origin,
    test_origin,
    test_page_cross_origin_frame,
    type_hint,
):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=test_page_cross_origin_frame,
        wait="complete",
    )

    evaluate_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(new_context["context"]),
        await_promise=False,
    )

    # Create a sandbox
    evaluate_in_sandbox_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(new_context["context"], "sandbox"),
        await_promise=False,
    )

    result = await bidi_session.script.get_realms(context=new_context["context"])
    recursive_compare(
        [
            {
                "context": new_context["context"],
                "origin": test_origin,
                "realm": evaluate_result["realm"],
                "type": "window",
            },
            {
                "context": new_context["context"],
                "origin": test_origin,
                "realm": evaluate_in_sandbox_result["realm"],
                "sandbox": "sandbox",
                "type": "window",
            },
        ],
        result,
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_context["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1
    frame_context = frames[0]["context"]

    evaluate_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(frame_context),
        await_promise=False,
    )

    # Create a sandbox in iframe
    evaluate_in_sandbox_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(frame_context, "sandbox"),
        await_promise=False,
    )

    result = await bidi_session.script.get_realms(context=frame_context)
    recursive_compare(
        [
            {
                "context": frame_context,
                "origin": test_alt_origin,
                "realm": evaluate_result["realm"],
                "type": "window",
            },
            {
                "context": frame_context,
                "origin": test_alt_origin,
                "realm": evaluate_in_sandbox_result["realm"],
                "sandbox": "sandbox",
                "type": "window",
            },
        ],
        result,
    )
