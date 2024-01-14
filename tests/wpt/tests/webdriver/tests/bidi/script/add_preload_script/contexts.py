import pytest

from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_top_context_with_iframes(
    bidi_session, add_preload_script, new_tab,
        inline, iframe, domain):

    iframe_content = f"<div>{domain}</div>"
    url = inline(f"{iframe(iframe_content, domain=domain)}")

    await add_preload_script(
        function_declaration="() => { window.bar='foo'; }",
        contexts=[new_tab["context"]])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete",
    )

    # Check that preload script applied the changes to the context
    result = await bidi_session.script.evaluate(
        expression="window.bar",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "foo"}

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])

    assert len(contexts[0]["children"]) == 1
    frame_context = contexts[0]["children"][0]

    # Check that preload script applied the changes to the iframe
    result = await bidi_session.script.evaluate(
        expression="window.bar",
        target=ContextTarget(frame_context["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "foo"}


@pytest.mark.asyncio
@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_page_script_context_isolation(bidi_session, add_preload_script,
                                             top_context, type_hint,
                                             test_page):
    await add_preload_script(function_declaration="() => { window.baz = 42; }",
                             contexts=[top_context['context']])

    new_context = await bidi_session.browsing_context.create(
        type_hint=type_hint)

    # Navigate both contexts to ensure preload script is triggered
    await bidi_session.browsing_context.navigate(
        context=top_context['context'],
        url=test_page,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=test_page,
        wait="complete",
    )

    # Check that preload script applied the changes to the context
    result = await bidi_session.script.evaluate(
        expression="window.baz",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )
    assert result == {"type": "number", "value": 42}

    # Check that preload script did *not* apply the changes to the other context
    result = await bidi_session.script.evaluate(
        expression="window.baz",
        target=ContextTarget(new_context["context"]),
        await_promise=True,
    )
    assert result == {type: "undefined"}


@pytest.mark.asyncio
async def test_identical_contexts(
        bidi_session, add_preload_script, new_tab,
        inline):

    url = inline(f"<div>test</div>")

    await add_preload_script(
        function_declaration="() => { window.foo = window.foo ? window.foo + 1 : 1; }",
        contexts=[new_tab["context"], new_tab["context"]])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete",
    )

    # Check that preload script applied the changes to the context only once
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "number", "value": "1"}
