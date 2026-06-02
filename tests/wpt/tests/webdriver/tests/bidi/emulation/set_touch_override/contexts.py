import pytest
from . import MAX_TOUCHES_PER_CONTEXT, MAX_TOUCHES_PER_USER_CONTEXT, \
    MAX_TOUCHES_GLOBAL

pytestmark = pytest.mark.asyncio


async def test_contexts_isolation(bidi_session, top_context,
                                  get_max_touch_points,
                                  initial_max_touch_points):
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab")

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_CONTEXT,
        contexts=[top_context["context"]])
    assert await get_max_touch_points(top_context) == MAX_TOUCHES_PER_CONTEXT
    assert await get_max_touch_points(
        another_context) == initial_max_touch_points

    yet_another_context = await bidi_session.browsing_context.create(
        type_hint="tab")
    assert await get_max_touch_points(
        yet_another_context) == initial_max_touch_points

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        contexts=[top_context["context"]])
    assert await get_max_touch_points(top_context) == initial_max_touch_points
    assert await get_max_touch_points(
        another_context) == initial_max_touch_points
    assert await get_max_touch_points(
        yet_another_context) == initial_max_touch_points


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
async def test_frame(bidi_session, url, get_max_touch_points, top_context,
                     create_iframe, domain, initial_max_touch_points):
    iframe_id = await create_iframe(top_context, url('/', domain=domain))

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_CONTEXT,
        contexts=[top_context["context"]])
    assert await get_max_touch_points(iframe_id) == MAX_TOUCHES_PER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        contexts=[top_context["context"]])
    assert await get_max_touch_points(iframe_id) == initial_max_touch_points


async def test_overrides_user_contexts(bidi_session, get_max_touch_points,
                                       affected_user_context,
                                       initial_max_touch_points):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_CONTEXT,
        contexts=[affected_context["context"]])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_USER_CONTEXT,
        user_contexts=[affected_user_context])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        contexts=[affected_context["context"]])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_USER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        user_contexts=[affected_user_context])
    assert await get_max_touch_points(
        affected_context) == initial_max_touch_points


async def test_overrides_global(bidi_session, get_max_touch_points,
                                affected_user_context,
                                initial_max_touch_points):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_CONTEXT,
        contexts=[affected_context["context"]])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_GLOBAL)
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        contexts=[affected_context["context"]])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_GLOBAL

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None)
    assert await get_max_touch_points(
        affected_context) == initial_max_touch_points
