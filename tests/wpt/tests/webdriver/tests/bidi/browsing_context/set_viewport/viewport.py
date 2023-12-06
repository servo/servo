import pytest
from webdriver.bidi.undefined import UNDEFINED

from ... import get_viewport_dimensions


@pytest.mark.asyncio
async def test_set_viewport(bidi_session, new_tab):
    test_viewport = {"width": 250, "height": 300}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport


@pytest.mark.asyncio
async def test_undefined_viewport(bidi_session, inline, new_tab):
    test_viewport = {"width": 499, "height": 599}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=UNDEFINED)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport


@pytest.mark.asyncio
@pytest.mark.parametrize("width, height", [
    (250, 300),
    (500, 300),
    (250, 600),
    (500, 600)
], ids=["none", "width", "height", "both"])
async def test_modified_dimensions(bidi_session, inline, new_tab, width, height):
    start_viewport = {"width": 250, "height": 300}

    assert await get_viewport_dimensions(bidi_session, new_tab) != start_viewport

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=start_viewport)

    assert await get_viewport_dimensions(bidi_session, new_tab) == start_viewport

    modified_viewport = {"width": width, "height": height}
    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=modified_viewport)

    assert await get_viewport_dimensions(bidi_session, new_tab) == modified_viewport


@pytest.mark.asyncio
async def test_reset_to_default(bidi_session, inline, new_tab):
    original_viewport = await get_viewport_dimensions(bidi_session, new_tab)

    test_viewport = {"width": 666, "height": 333}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport
    )

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=None
    )
    assert await get_viewport_dimensions(bidi_session, new_tab) == original_viewport


@pytest.mark.asyncio
async def test_specific_context(bidi_session, inline, new_tab, top_context):
    original_viewport = await get_viewport_dimensions(bidi_session, top_context)

    test_viewport = {"width": 333, "height": 666}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport
    )

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport
    assert await get_viewport_dimensions(bidi_session, top_context) == original_viewport


@pytest.mark.parametrize("protocol,parameters", [
    ("http", ""),
    ("https", ""),
    ("https", {"pipe": "header(Cross-Origin-Opener-Policy,same-origin)"})
], ids=[
    "http",
    "https",
    "https coop"
])
@pytest.mark.asyncio
async def test_persists_on_navigation(bidi_session, new_tab, inline, protocol, parameters):
    test_viewport = {"width": 499, "height": 599}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport

    url = inline("<div>foo</div>", parameters=parameters, protocol=protocol)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport

    url = inline("<div>bar</div>", parameters=parameters, protocol=protocol, domain="alt")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport


@pytest.mark.asyncio
async def test_persists_on_reload(bidi_session, inline, new_tab):
    test_viewport = {"width": 499, "height": 599}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport

    await bidi_session.browsing_context.reload(
        context=new_tab["context"], wait="complete"
    )

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport
