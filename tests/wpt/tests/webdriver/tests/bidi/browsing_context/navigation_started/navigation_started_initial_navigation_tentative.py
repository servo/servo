import pytest

from webdriver.bidi.modules.script import ContextTarget

from .. import assert_navigation_info

pytestmark = pytest.mark.asyncio

NAVIGATION_STARTED_EVENT = "browsingContext.navigationStarted"


# Tentative: https://github.com/web-platform-tests/wpt/issues/47942

@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, subscribe_events, wait_for_event,
      wait_for_future_safe, type_hint):
    await subscribe_events(events=[NAVIGATION_STARTED_EVENT])

    on_entry = wait_for_event(NAVIGATION_STARTED_EVENT)
    top_level_context = await bidi_session.browsing_context.create(
        type_hint="tab")
    navigation_info = await wait_for_future_safe(on_entry)
    assert_navigation_info(
        navigation_info,
        {
            "context": top_level_context["context"],
            "url": "about:blank",
        },
    )


async def test_window_open(bidi_session, subscribe_events, wait_for_event,
      wait_for_future_safe, top_context):
    await subscribe_events(events=[NAVIGATION_STARTED_EVENT])

    on_entry = wait_for_event(NAVIGATION_STARTED_EVENT)

    await bidi_session.script.evaluate(
        expression="""window.open('about:blank');""",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    navigation_info = await wait_for_future_safe(on_entry)
    assert_navigation_info(
        navigation_info,
        {
            "url": "about:blank",
        },
    )
    assert navigation_info["navigation"] is not None

    # Retrieve all contexts to get the context for the new window.
    contexts = await bidi_session.browsing_context.get_tree()
    assert navigation_info["context"] == contexts[-1]["context"]
