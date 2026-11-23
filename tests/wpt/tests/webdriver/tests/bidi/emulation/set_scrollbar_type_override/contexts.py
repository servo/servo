import pytest

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session, new_tab, top_context, get_scrollbar_width, default_scrollbar_width
):
    # Verify baseline state.
    assert await get_scrollbar_width(top_context) == default_scrollbar_width
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width

    # Apply override only to new_tab.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_scrollbar_width(new_tab) == 0
    assert await get_scrollbar_width(top_context) == default_scrollbar_width

    # Create another context and verify isolation.
    another_context = await bidi_session.browsing_context.create(type_hint="tab")
    assert await get_scrollbar_width(another_context) == default_scrollbar_width

    # Clear context-level override.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width
    assert await get_scrollbar_width(top_context) == default_scrollbar_width
    assert await get_scrollbar_width(another_context) == default_scrollbar_width


async def test_multiple_contexts(
    bidi_session, new_tab, top_context, get_scrollbar_width, default_scrollbar_width
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Apply override to multiple contexts simultaneously.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"], new_context["context"]],
        scrollbar_type="overlay",
    )
    assert await get_scrollbar_width(new_tab) == 0
    assert await get_scrollbar_width(new_context) == 0
    assert await get_scrollbar_width(top_context) == default_scrollbar_width

    # Clear override on both contexts.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"], new_context["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width
    assert await get_scrollbar_width(new_context) == default_scrollbar_width


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    new_tab,
    url,
    create_iframe,
    domain,
    get_scrollbar_width,
    default_scrollbar_width,
):
    # Apply override to new_tab.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_scrollbar_width(new_tab) == 0

    # Verify inheritance in child iframe.
    iframe = await create_iframe(new_tab, url("/", domain=domain))
    assert await get_scrollbar_width(iframe) == 0

    # Clear override on new_tab and verify iframe falls back.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width
    assert await get_scrollbar_width(iframe) == default_scrollbar_width


async def test_overrides_user_contexts(
    bidi_session, affected_user_context, get_scrollbar_width, default_scrollbar_width
):
    context_in_user_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context
    )

    # Set context-level and user-context-level overrides.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[context_in_user_context["context"]],
        scrollbar_type="classic",
    )
    await bidi_session.emulation.set_scrollbar_type_override(
        user_contexts=[affected_user_context],
        scrollbar_type="overlay",
    )

    # Context-level override takes precedence.
    assert await get_scrollbar_width(context_in_user_context) > 0

    # Clearing context-level setting falls back to user-context setting.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[context_in_user_context["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(context_in_user_context) == 0

    # Clearing user-context setting reverts to default.
    await bidi_session.emulation.set_scrollbar_type_override(
        user_contexts=[affected_user_context],
        scrollbar_type=None,
    )
    assert (
        await get_scrollbar_width(context_in_user_context)
        == default_scrollbar_width
    )


async def test_overrides_global(
    bidi_session, new_tab, get_scrollbar_width, default_scrollbar_width
):
    # Set context-level and global-level overrides.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    await bidi_session.emulation.set_scrollbar_type_override(
        scrollbar_type="overlay",
    )

    # Context-level override takes precedence over global.
    assert await get_scrollbar_width(new_tab) > 0

    # Clearing context-level setting falls back to global setting.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == 0

    # Clearing global setting reverts to default.
    await bidi_session.emulation.set_scrollbar_type_override(
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_existing_iframe(
    bidi_session,
    new_tab,
    inline,
    iframe,
    domain,
    get_scrollbar_width,
    default_scrollbar_width,
):
    # Load a page with an iframe before setting the override.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(iframe("<div>foo</div>", domain=domain)),
        wait="complete",
    )
    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    existing_iframe = contexts[0]["children"][0]

    # Apply overrides to new_tab, and verify that they apply to the iframe.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_scrollbar_width(existing_iframe) == 0

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    assert await get_scrollbar_width(existing_iframe) > 0

    # Clear override on new_tab and verify the iframe falls back.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(existing_iframe) == default_scrollbar_width


@pytest.mark.parametrize(
    "content_height", ["1000px", "10px"], ids=["scrollable", "not_scrollable"]
)
@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_existing_iframe_own_scrollbar(
    bidi_session, new_tab, inline, iframe, domain, content_height
):
    # The iframe's document has a vertical scrollbar, whether or not its content
    # overflows. It belongs to the iframe's viewport rather than to an element,
    # and changing its type changes the width that the iframe's document is laid
    # out in.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(
            iframe(
                "<style>html { overflow-y: scroll; }</style>"
                f"<div style='height: {content_height}'>foo</div>",
                domain=domain,
            )
        ),
        wait="complete",
    )
    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    existing_iframe = contexts[0]["children"][0]

    async def evaluate_in_iframe(expression):
        result = await bidi_session.script.call_function(
            function_declaration=f"() => {expression}",
            target=ContextTarget(existing_iframe["context"]),
            await_promise=False,
        )
        return result["value"]

    async def get_viewport_scrollbar_width():
        return await evaluate_in_iframe(
            "window.innerWidth - document.documentElement.clientWidth"
        )

    async def assert_laid_out_for_viewport():
        # The root element fills the width of the viewport, without its scrollbar.
        assert await evaluate_in_iframe(
            "document.documentElement.getBoundingClientRect().width"
        ) == await evaluate_in_iframe("document.documentElement.clientWidth")

    default_width = await get_viewport_scrollbar_width()
    await assert_laid_out_for_viewport()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_viewport_scrollbar_width() == 0
    await assert_laid_out_for_viewport()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    assert await get_viewport_scrollbar_width() > 0
    await assert_laid_out_for_viewport()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_viewport_scrollbar_width() == default_width
    await assert_laid_out_for_viewport()


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_navigation(
    bidi_session,
    new_tab,
    inline,
    domain,
    get_scrollbar_width,
    default_scrollbar_width,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    # Apply override to new_tab, and verify it persists after navigation.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<div>bar</div>", domain=domain),
        wait="complete",
    )
    assert await get_scrollbar_width(new_tab) == 0

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    assert await get_scrollbar_width(new_tab) > 0

    # Clear override on new_tab and verify the navigated page falls back.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width
