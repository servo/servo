import pytest
from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException

from ... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize(
    "expression",
    [
        "document.querySelector('iframe').contentWindow",
        "frames[0].window",
        "frames[0].self",
    ],
)
async def test_iframe_window(bidi_session, top_context, inline, iframe, domain, expression):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(iframe("<div>foo</div>", domain=domain)),
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    iframe_context = contexts[0]["children"][0]

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == {
        "type": "window",
        "value": {"context": iframe_context["context"]},
    }


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_windows_in_containers(bidi_session, top_context, inline, iframe, domain):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(iframe("<div>foo</div>", domain=domain)),
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    iframe_context = contexts[0]["children"][0]

    result = await bidi_session.script.evaluate(
        expression="({ windows: [window, new Set([frames[0]])], map: new Map([[frames[0], window]]) })",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    top_window = {"type": "window", "value": {"context": top_context["context"]}}
    iframe_window = {"type": "window", "value": {"context": iframe_context["context"]}}
    recursive_compare(
        {
            "type": "object",
            "value": [
                ["windows", {
                    "type": "array",
                    "value": [top_window, {"type": "set", "value": [iframe_window]}],
                }],
                ["map", {"type": "map", "value": [[iframe_window, top_window]]}],
            ],
        },
        result,
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_thrown_window(bidi_session, top_context, inline, iframe, domain):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(iframe("<div>foo</div>", domain=domain)),
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    iframe_context = contexts[0]["children"][0]

    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression="throw [frames[0]]",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )

    recursive_compare(
        {
            "type": "array",
            "value": [{"type": "window", "value": {"context": iframe_context["context"]}}],
        },
        exception.value.result["exceptionDetails"]["exception"],
    )


async def test_popup_window(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="window.open()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare({"type": "window", "value": {"context": any_string}}, result)
    popup_context = result["value"]["context"]
    assert popup_context != top_context["context"]

    contexts = await bidi_session.browsing_context.get_tree(root=popup_context)
    assert len(contexts) == 1
    assert contexts[0]["parent"] is None

    await bidi_session.browsing_context.close(context=popup_context)
