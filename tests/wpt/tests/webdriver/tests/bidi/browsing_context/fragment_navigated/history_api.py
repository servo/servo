import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import any_int, recursive_compare

pytestmark = pytest.mark.asyncio

EMPTY_PAGE = "/webdriver/tests/bidi/support/empty.html"
FRAGMENT_NAVIGATED_EVENT = "browsingContext.fragmentNavigated"


@pytest.mark.parametrize(
    "hash_before, hash_after",
    [
        ("", "#foo"),
        ("#foo", "#bar"),
        ("#foo", "#foo"),
        ("#bar", ""),
    ]
)
async def test_history_push_state(
    bidi_session, new_tab, url, subscribe_events, wait_for_event, hash_before, hash_after
):
    target_context = new_tab["context"]

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE + hash_before), wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT])

    on_frame_navigated = wait_for_event(FRAGMENT_NAVIGATED_EVENT)

    target_url = url(EMPTY_PAGE + hash_after)

    await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(url) => {
            history.pushState(null, null, url);
        }""",
        arguments=[
            {"type": "string", "value": target_url},
        ],
        await_promise=False,
        target=ContextTarget(target_context),
    )

    recursive_compare(
        {
            'context': target_context,
            'timestamp': any_int,
            'url': target_url
        },
        await on_frame_navigated,
    )
