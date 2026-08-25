import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"


def find_context_info(contexts, context):
    return next(
        (
            context_info
            for context_info in contexts
            if context_info["context"] == context
        ),
        None,
    )


@pytest.mark.parametrize(
    "features",
    [None, "", "popup", "noopener", "noreferrer"],
)
async def test_window_open(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    subscribe_events,
    inline,
    features,
):
    top_level_context = await bidi_session.browsing_context.create(type_hint="tab")

    await subscribe_events([CONTEXT_CREATED_EVENT])
    on_created = wait_for_event(CONTEXT_CREATED_EVENT)

    await bidi_session.script.evaluate(
        expression=f"""window.open("{inline("")}", "_blank", "{features}");""",
        target=ContextTarget(top_level_context["context"]),
        await_promise=False,
    )

    target_context = await wait_for_future_safe(on_created)

    contexts = await bidi_session.browsing_context.get_tree()

    assert len(contexts) == 3

    found_context = find_context_info(contexts, target_context["context"])
    assert found_context["originalOpener"] == top_level_context["context"]


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_different_origins(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    subscribe_events,
    inline,
    domain,
):
    top_level_context = await bidi_session.browsing_context.create(type_hint="tab")

    await subscribe_events([CONTEXT_CREATED_EVENT])
    on_created = wait_for_event(CONTEXT_CREATED_EVENT)

    url = inline("", domain=domain)

    await bidi_session.script.evaluate(
        expression=f"""window.open("{url}", "_blank");""",
        target=ContextTarget(top_level_context["context"]),
        await_promise=False,
    )

    target_context = await wait_for_future_safe(on_created)

    contexts = await bidi_session.browsing_context.get_tree()

    assert len(contexts) == 3
    found_context = find_context_info(contexts, target_context["context"])
    assert found_context["originalOpener"] == top_level_context["context"]


async def test_with_closed_original_context(
    bidi_session,
    inline,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    top_level_context = await bidi_session.browsing_context.create(type_hint="tab")

    await subscribe_events([CONTEXT_CREATED_EVENT])
    on_created = wait_for_event(CONTEXT_CREATED_EVENT)

    await bidi_session.script.evaluate(
        expression=f"""window.open("{inline("")}", "_blank", "");""",
        target=ContextTarget(top_level_context["context"]),
        await_promise=False,
    )

    target_context = await wait_for_future_safe(on_created)

    # Close the context which initiated opening the window.
    await bidi_session.browsing_context.close(context=top_level_context["context"])

    contexts = await bidi_session.browsing_context.get_tree()

    assert len(contexts) == 2

    found_context = find_context_info(contexts, target_context["context"])
    assert found_context["originalOpener"] == top_level_context["context"]
