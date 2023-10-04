from pathlib import Path

import pytest

from . import reload_and_assert


pytestmark = pytest.mark.asyncio

PNG_BLACK_DOT = "/webdriver/tests/bidi/browsing_context/support/black_dot.png"


@pytest.mark.parametrize("hash", [False, True], ids=["without hash", "with hash"])
async def test_reload(bidi_session, inline, new_tab, hash):
    url = inline("""<div id="foo""")
    if hash:
        url += "#foo"

    navigate_result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete"
    )

    reload_and_assert(
        bidi_session,
        new_tab,
        last_navigation=navigate_result["navigation"],
        url=url
    )


@pytest.mark.parametrize(
    "url",
    [
        "about:blank",
        "data:text/html,<p>foo</p>",
        f'{Path(__file__).parents[1].as_uri()}/support/empty.html',
    ],
    ids=[
        "about:blank",
        "data url",
        "file url",
    ],
)
async def test_reload_special_protocols(bidi_session, new_tab, url):
    navigate_result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete"
    )

    reload_and_assert(
        bidi_session,
        new_tab,
        last_navigation=navigate_result["navigation"],
        url=url
    )


async def test_image(bidi_session, new_tab, url):
    navigate_result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PNG_BLACK_DOT),
        wait="complete"
    )

    reload_and_assert(
        bidi_session,
        new_tab,
        last_navigation=navigate_result["navigation"],
        url=url
    )
