import pytest
from webdriver.bidi.modules.script import ContextTarget

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio

CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_original_opener_context_create(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):

    await subscribe_events([CONTEXT_CREATED_EVENT])
    on_created = wait_for_event(CONTEXT_CREATED_EVENT)

    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    context_info = await wait_for_future_safe(on_created)

    assert_browsing_context(
        context_info,
        context=top_level_context["context"],
        original_opener=None,
        url="about:blank",
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("features, returns_window", [
    ("", True),
    ("popup", True),
    ("noopener", False),
    ("noreferrer", False)
]
)
async def test_original_opener_window_open(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, inline,
                                           type_hint, domain, features, returns_window):

    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await subscribe_events([CONTEXT_CREATED_EVENT])
    on_created = wait_for_event(CONTEXT_CREATED_EVENT)

    url = inline("", domain=domain)

    result = await bidi_session.script.evaluate(
        expression=f"""window.open("{url}", "_blank", "{features}");""",
        target=ContextTarget(top_level_context["context"]),
        await_promise=False)

    context_info = await wait_for_future_safe(on_created)

    # We use None here as evaluate not always returns value.
    context = None
    if returns_window:
        context = result["value"]["context"]

    assert_browsing_context(
        context_info,
        context=context,
        original_opener=top_level_context["context"],
        url="about:blank",
    )
