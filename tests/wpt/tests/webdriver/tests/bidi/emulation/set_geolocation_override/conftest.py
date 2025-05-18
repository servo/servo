import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

from ... import get_context_origin, remote_mapping_to_dict


@pytest_asyncio.fixture
async def get_current_geolocation(bidi_session, configuration):
    async def get_current_geolocation(context):
        # Per geolocation spec, the geolocation coordinates are returned
        # only for an active browsing context. It might be required to
        # re-activate the previously active tab in the test.
        await bidi_session.browsing_context.activate(context=context["context"])

        result = await bidi_session.script.call_function(
            function_declaration="""(multiplier) =>
                new Promise(
                    resolve => window.navigator.geolocation.getCurrentPosition(
                        position => resolve(position.coords.toJSON()),
                        error => resolve({code: error.code}),
                        {timeout: 500 * multiplier}
                ))
            """,
            arguments=[{"type": "number", "value": configuration["timeout_multiplier"]}],
            target=ContextTarget(context["context"]),
            await_promise=True,
        )

        return remote_mapping_to_dict(result["value"])

    return get_current_geolocation


@pytest_asyncio.fixture
async def set_geolocation_permission(bidi_session):
    data_to_cleanup = {}

    async def set_geolocation_permission(context, user_context="default"):
        nonlocal data_to_cleanup

        origin = await get_context_origin(bidi_session, context)

        data_to_cleanup = {
            "origin": origin,
            "user_context": user_context,
        }

        await bidi_session.permissions.set_permission(
            descriptor={"name": "geolocation"},
            state="granted",
            origin=origin,
            user_context=user_context,
        )

    yield set_geolocation_permission

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="prompt",
        origin=data_to_cleanup["origin"],
        user_context=data_to_cleanup["user_context"],
    )
