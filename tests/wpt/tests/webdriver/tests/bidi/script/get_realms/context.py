import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare


@pytest.mark.asyncio
async def test_context(
    bidi_session,
    test_alt_origin,
    test_origin,
    test_page_cross_origin_frame,
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=test_page_cross_origin_frame,
        wait="complete",
    )

    # Evaluate to get realm id
    new_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(new_context["context"]),
        await_promise=False,
    )

    result = await bidi_session.script.get_realms(context=new_context["context"])

    recursive_compare(
        [
            {
                "context": new_context["context"],
                "origin": test_origin,
                "realm": new_context_result["realm"],
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

    # Evaluate to get realm id
    frame_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1 + 2",
        target=ContextTarget(frame_context),
        await_promise=False,
    )

    result = await bidi_session.script.get_realms(context=frame_context)

    recursive_compare(
        [
            {
                "context": frame_context,
                "origin": test_alt_origin,
                "realm": frame_context_result["realm"],
                "type": "window",
            },
        ],
        result,
    )
