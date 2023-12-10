import pytest
from webdriver.bidi.modules.script import ContextTarget, SerializationOptions
from ... import recursive_compare
from .. import REMOTE_VALUES


@pytest.mark.asyncio
@pytest.mark.parametrize("expression, expected", REMOTE_VALUES)
async def test_remote_values(bidi_session, top_context, expression, expected):
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        serialization_options=SerializationOptions(max_object_depth=1),
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize("await_promise", [True, False])
async def test_window_context_top_level(bidi_session, top_context, await_promise):
    result = await bidi_session.script.evaluate(
        expression="window",
        target=ContextTarget(top_context["context"]),
        await_promise=await_promise,
        serialization_options=SerializationOptions(max_object_depth=1),
    )

    recursive_compare(
        {
            "type": "window",
            "value": {
                "context": top_context["context"]
            }
        }, result)


@pytest.mark.asyncio
@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("await_promise", [True, False])
async def test_window_context_iframe_window(
        bidi_session, top_context, inline, domain, await_promise):
    frame_url = inline("<div>foo</div>")
    url = inline(f"<iframe src='{frame_url}'></iframe>", domain=domain)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree()
    iframe_context = all_contexts[0]["children"][0]

    result = await bidi_session.script.evaluate(
        expression="window",
        target=ContextTarget(iframe_context["context"]),
        await_promise=await_promise,
        serialization_options=SerializationOptions(max_object_depth=1),
    )

    recursive_compare(
        {
            "type": "window",
            "value": {
                "context": iframe_context["context"]
            }
        }, result)


@pytest.mark.asyncio
@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("await_promise", [True, False])
async def test_window_context_iframe_content_window(
        bidi_session, top_context, inline, domain, await_promise):

    frame_url = inline("<div>foo</div>")
    url = inline(f"<iframe src='{frame_url}'></iframe>", domain=domain)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree()
    iframe_context = all_contexts[0]["children"][0]

    # This is equivalent to `document.getElementsByTagName("iframe")[0].conten
    result = await bidi_session.script.evaluate(
        expression="window.frames[0]",
        target=ContextTarget(top_context["context"]),
        await_promise=await_promise,
    )

    recursive_compare(
        {
            "type": "window",
            "value": {
                "context": iframe_context["context"]
            }
        }, result)


@pytest.mark.asyncio
@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("await_promise", [True, False])
async def test_window_context_same_id_after_navigation(bidi_session,
                                                       top_context,
                                                       inline,
                                                       domain,
                                                       await_promise):

    defaultOrigin = inline(f"{domain}")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=defaultOrigin, wait="complete")

    url = inline(f"{domain}", domain=domain)

    result = await bidi_session.script.evaluate(
        expression="window",
        target=ContextTarget(top_context["context"]),
        await_promise=await_promise,
        serialization_options=SerializationOptions(max_object_depth=1),
    )

    original_context = result['value']['context']

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete")

    result = await bidi_session.script.evaluate(
        expression="window",
        target=ContextTarget(top_context["context"]),
        await_promise=await_promise,
        serialization_options=SerializationOptions(max_object_depth=1),
    )

    navigated_context_id = result['value']['context']

    assert navigated_context_id == original_context
