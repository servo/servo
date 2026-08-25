import pytest

import webdriver.bidi.error as error


pytestmark = pytest.mark.asyncio


async def test_top_level_contexts(
    bidi_session, current_url, wait_for_url, top_context, new_tab, inline
):
    pages = [
        inline("<div>page 1</div>"),
        inline("<div>page 2</div>"),
    ]
    for page in pages:
        for context in [top_context["context"], new_tab["context"]]:
            await bidi_session.browsing_context.navigate(
                context=context, url=page, wait="complete"
            )
            assert await current_url(context) == page

    await bidi_session.browsing_context.traverse_history(
        context=new_tab["context"], delta=-1
    )

    await wait_for_url(top_context["context"], pages[1])
    await wait_for_url(new_tab["context"], pages[0])
