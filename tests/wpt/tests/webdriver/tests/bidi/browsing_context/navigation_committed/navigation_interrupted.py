import pytest

from .. import assert_navigation_info


pytestmark = pytest.mark.asyncio

NAVIGATION_COMMITTED_EVENT = "browsingContext.navigationCommitted"
PAGE_EMPTY = "/webdriver/tests/bidi/browsing_context/support/empty.html"


@pytest.mark.parametrize(
    "script",
    [
        "<script>window.location='{url}'</script>",
        """<script>window.addEventListener('DOMContentLoaded', () => {{
            window.location = '{url}';
        }});</script>""",
        """<script>window.addEventListener('load', () => {{
            window.location = '{url}';
       }});</script>""",
    ],
    ids=[
        "Interrupted immediately",
        "Interrupted on DOMContentLoaded",
        "Interrupted on load",
    ],
)
async def test_multiple_events_for_interrupted_navigation(
    bidi_session,
    subscribe_events,
    new_tab,
    url,
    inline,
    wait_for_events,
    script,
):
    url_after = url(PAGE_EMPTY)
    url_before = inline(script.format(url=url_after))

    await subscribe_events([NAVIGATION_COMMITTED_EVENT], contexts=[new_tab["context"]])
    with wait_for_events([NAVIGATION_COMMITTED_EVENT]) as waiter:
        result = await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=url_before, wait="none"
        )
        events = await waiter.get_events(
            lambda events: len(events) >= 2
        )

        assert len(events) == 2
        assert_navigation_info(
            events[0][1],
            {
                "context": new_tab["context"],
                "url": url_before,
            },
        )
        assert_navigation_info(
            events[1][1],
            {
                "context": new_tab["context"],
                "url": url_after,
            },
        )
