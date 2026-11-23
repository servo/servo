import pytest

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_scrollbar_type_overlay(
    bidi_session, new_tab, get_scrollbar_width, default_scrollbar_width
):
    # Verify baseline state.
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width

    # Apply overlay scrollbar type override.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_scrollbar_width(new_tab) == 0

    # Reset override to verify fallback.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width


async def test_scrollbar_type_classic(
    bidi_session, new_tab, get_scrollbar_width, default_scrollbar_width
):
    # Apply classic scrollbar type override.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    assert await get_scrollbar_width(new_tab) > 0

    # Reset override to verify fallback.
    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_scrollbar_width(new_tab) == default_scrollbar_width


async def test_existing_page_scrollbar(bidi_session, new_tab, inline):
    # A page that is taller than the viewport, so it has a vertical scrollbar
    # before any override is set.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<div style='height: 10000px'>foo</div>"),
        wait="complete",
    )

    async def get_page_scrollbar_width():
        result = await bidi_session.script.evaluate(
            expression="window.innerWidth - document.documentElement.clientWidth",
            target=ContextTarget(new_tab["context"]),
            await_promise=False,
        )
        return result["value"]

    default_page_scrollbar_width = await get_page_scrollbar_width()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_page_scrollbar_width() == 0

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    assert await get_page_scrollbar_width() > 0

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_page_scrollbar_width() == default_page_scrollbar_width


@pytest.mark.parametrize(
    "content_height", ["1000px", "10px"], ids=["scrollable", "not_scrollable"]
)
async def test_existing_element_scrollbar(bidi_session, new_tab, inline, content_height):
    # An element that has a vertical scrollbar before any override is set, whether
    # or not its content overflows. Changing the scrollbar's type changes the
    # width that the element's content is laid out in.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(
            "<div id='scroller' style='overflow-y: scroll; width: 200px; height: 100px'>"
            f"<div id='content' style='height: {content_height}'>foo</div></div>"
        ),
        wait="complete",
    )

    async def evaluate(expression):
        result = await bidi_session.script.evaluate(
            expression=expression,
            target=ContextTarget(new_tab["context"]),
            await_promise=False,
        )
        return result["value"]

    async def get_element_scrollbar_width():
        return await evaluate(
            "(scroller => scroller.offsetWidth - scroller.clientWidth)(document.getElementById('scroller'))"
        )

    async def assert_laid_out_for_scrollbar():
        # The content fills the width of the element, without its scrollbar.
        assert await evaluate(
            "document.getElementById('content').getBoundingClientRect().width"
        ) == await evaluate("document.getElementById('scroller').clientWidth")

    default_width = await get_element_scrollbar_width()
    await assert_laid_out_for_scrollbar()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    assert await get_element_scrollbar_width() == 0
    await assert_laid_out_for_scrollbar()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    assert await get_element_scrollbar_width() > 0
    await assert_laid_out_for_scrollbar()

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type=None,
    )
    assert await get_element_scrollbar_width() == default_width
    await assert_laid_out_for_scrollbar()
