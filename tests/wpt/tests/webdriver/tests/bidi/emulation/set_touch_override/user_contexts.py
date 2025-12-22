import pytest
from . import MAX_TOUCHES_PER_USER_CONTEXT, MAX_TOUCHES_GLOBAL

pytestmark = pytest.mark.asyncio


async def test_isolation(bidi_session, get_max_touch_points,
                         affected_user_context, not_affected_user_context,
                         initial_max_touch_points):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    not_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=not_affected_user_context)

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_USER_CONTEXT,
        user_contexts=[affected_user_context])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_USER_CONTEXT
    assert await get_max_touch_points(
        not_affected_context) == initial_max_touch_points

    another_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    another_not_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=not_affected_user_context)
    assert await get_max_touch_points(
        another_affected_context) == MAX_TOUCHES_PER_USER_CONTEXT
    assert await get_max_touch_points(
        another_not_affected_context) == initial_max_touch_points

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        user_contexts=[affected_user_context])

    assert await get_max_touch_points(
        affected_context) == initial_max_touch_points
    assert await get_max_touch_points(
        not_affected_context) == initial_max_touch_points
    assert await get_max_touch_points(
        another_affected_context) == initial_max_touch_points
    assert await get_max_touch_points(
        another_not_affected_context) == initial_max_touch_points


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
async def test_frame(bidi_session, url, get_max_touch_points, create_iframe,
                     domain, affected_user_context, initial_max_touch_points):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    iframe_id = await create_iframe(affected_context, url('/', domain=domain))

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_USER_CONTEXT,
        user_contexts=[affected_user_context])

    assert await get_max_touch_points(iframe_id) == MAX_TOUCHES_PER_USER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        user_contexts=[affected_user_context])

    assert await get_max_touch_points(iframe_id) == initial_max_touch_points


async def test_overrides_global(bidi_session, get_max_touch_points,
                                affected_user_context,
                                initial_max_touch_points):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_PER_USER_CONTEXT,
        user_contexts=[affected_user_context])
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_USER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=MAX_TOUCHES_GLOBAL)
    assert await get_max_touch_points(
        affected_context) == MAX_TOUCHES_PER_USER_CONTEXT

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None,
        user_contexts=[affected_user_context])
    assert await get_max_touch_points(affected_context) == MAX_TOUCHES_GLOBAL

    await bidi_session.emulation.set_touch_override(
        max_touch_points=None)
    assert await get_max_touch_points(
        affected_context) == initial_max_touch_points
