from webdriver.bidi.modules.script import ContextTarget

from ... import remote_mapping_to_dict


TEST_COORDINATES = {"latitude": 10, "longitude": 15, "accuracy": 0.5}


async def get_current_geolocation(bidi_session, context):
    # Per geolocation spec, the geolocation coordinates are returned
    # only for an active browsing context. It might be required to
    # re-activate the previously active tab in the test.
    await bidi_session.browsing_context.activate(context=context["context"])

    result = await bidi_session.script.call_function(
        function_declaration="""() =>
            new Promise(
                resolve => window.navigator.geolocation.getCurrentPosition(
                    position => resolve(position.coords.toJSON()),
                    error => resolve({code: error.code, message: error.message}),
                    {timeout: 500}
            ))
        """,
        target=ContextTarget(context["context"]),
        await_promise=True,
    )

    return remote_mapping_to_dict(result["value"])
