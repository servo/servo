from pathlib import Path

import pytest
from webdriver import error
from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll


pytestmark = pytest.mark.asyncio


async def test_delta_0(
    bidi_session, current_url, wait_for_url, wait_for_not_url, new_tab, inline
):
    pages = [
        inline("<div>page 1</div>"),
        inline("<div>page 2</div>"),
        inline("<div>page 3</div>"),
    ]
    for page in pages:
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=page, wait="complete"
        )
        assert await current_url(new_tab["context"]) == page

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=-1
    )
    await wait_for_url(new_tab["context"], pages[1])

    # With delta 0 no navigation has to happen
    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=0
    )
    with pytest.raises(error.TimeoutException):
        await wait_for_not_url(new_tab["context"], pages[1])


async def test_delta_forward_and_back(
    bidi_session, current_url, wait_for_url, new_tab, inline
):
    pages = [
        inline("<div>page 1</div>"),
        inline("<div>page 2</div>"),
        inline("<div>page 3</div>"),
    ]
    for page in pages:
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=page, wait="complete"
        )
        assert await current_url(new_tab["context"]) == page

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=-2
    )

    await wait_for_url(new_tab["context"], pages[0])

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=2
    )

    await wait_for_url(new_tab["context"], pages[2])


async def test_navigate_in_the_same_document(
    bidi_session, current_url, wait_for_url, new_tab, url
):
    page_url = "/webdriver/tests/bidi/browsing_context/support/empty.html"
    pages = [
        url(page_url),
        url(page_url + "#foo"),
        url(page_url + "#bar"),
    ]
    for page in pages:
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=page, wait="complete"
        )
        assert await current_url(new_tab["context"]) == page

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=-1
    )

    await wait_for_url(new_tab["context"], pages[1])

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=1
    )

    await wait_for_url(new_tab["context"], pages[2])


async def test_history_push_state(
    bidi_session, current_url, wait_for_url, new_tab, url
):
    page_url = url("/webdriver/tests/bidi/browsing_context/support/empty.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )
    assert await current_url(new_tab["context"]) == page_url

    pages = [
        f"{page_url}#foo",
        f"{page_url}#bar",
    ]
    for page in pages:
        await bidi_session.script.call_function(
            function_declaration="""(url) => {
                history.pushState(null, null, url);
            }""",
            arguments=[
                {"type": "string", "value": page},
            ],
            await_promise=False,
            target=ContextTarget(new_tab["context"]),
        )
        await wait_for_url(new_tab["context"], page)

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=-1
    )

    await wait_for_url(new_tab["context"], pages[0])

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=1
    )

    await wait_for_url(new_tab["context"], pages[1])


@pytest.mark.parametrize(
    "pages",
    [
        ["data:text/html,<p>foo</p>", "data:text/html,<p>bar</p>"],
        [
            f"{Path(__file__).parents[1].as_uri()}/support/empty.html",
            f"{Path(__file__).parents[1].as_uri()}/support/other.html",
        ],
    ],
    ids=[
        "data url",
        "file url",
    ],
)
async def test_navigate_special_protocols(
    bidi_session, current_url, wait_for_url, new_tab, pages
):
    for page in pages:
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=page, wait="complete"
        )
        assert await current_url(new_tab["context"]) == page

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=-1
    )

    await wait_for_url(new_tab["context"], pages[0])

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=1
    )

    await wait_for_url(new_tab["context"], pages[1])
