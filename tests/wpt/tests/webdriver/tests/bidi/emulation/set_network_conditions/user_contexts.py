import pytest

from . import OFFLINE_NETWORK_CONDITIONS

pytestmark = pytest.mark.asyncio


async def test_isolation(bidi_session, create_user_context,
        get_navigator_online, affected_user_context, not_affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    not_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=not_affected_user_context)

    assert await get_navigator_online(affected_context)
    assert await get_navigator_online(not_affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        user_contexts=[affected_user_context])

    assert not await get_navigator_online(affected_context)
    assert await get_navigator_online(not_affected_context)

    another_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    another_not_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=not_affected_user_context)
    assert not await get_navigator_online(another_affected_context)
    assert await get_navigator_online(another_not_affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        user_contexts=[affected_user_context])

    assert await get_navigator_online(affected_context)
    assert await get_navigator_online(not_affected_context)
    assert await get_navigator_online(another_affected_context)
    assert await get_navigator_online(another_not_affected_context)


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
async def test_frame(bidi_session, url, get_navigator_online,
        top_context, create_iframe, domain, affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    iframe_id = await create_iframe(affected_context, url('/', domain=domain))

    assert await get_navigator_online(iframe_id)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        user_contexts=[affected_user_context])

    assert not await get_navigator_online(iframe_id)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        user_contexts=[affected_user_context])

    assert await get_navigator_online(iframe_id)


async def test_overrides_global(bidi_session, get_navigator_online,
        affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    assert await get_navigator_online(affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        user_contexts=[affected_user_context])

    assert not await get_navigator_online(affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=None)

    assert not await get_navigator_online(affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS)

    assert not await get_navigator_online(affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        user_contexts=[affected_user_context])

    assert not await get_navigator_online(affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=None)

    assert await get_navigator_online(affected_context)


async def test_restores_to_global_when_removed(bidi_session,
        get_navigator_online,
        affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)

    assert await get_navigator_online(affected_context)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS,
        user_contexts=[affected_user_context])

    await bidi_session.emulation.set_network_conditions(
        network_conditions=OFFLINE_NETWORK_CONDITIONS)

    await bidi_session.emulation.set_network_conditions(
        network_conditions=None,
        user_contexts=[affected_user_context])

    assert not await get_navigator_online(affected_context)
