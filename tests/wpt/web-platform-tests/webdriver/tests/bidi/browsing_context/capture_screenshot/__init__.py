from webdriver.bidi.modules.script import ContextTarget

async def viewport_dimensions(bidi_session, context):
    """Get the dimensions of the context's viewport.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :returns: Tuple of (int, int) containing viewport width, viewport height.
    """
    result = await bidi_session.script.call_function(
        function_declaration="""() => {
        const {devicePixelRatio, innerHeight, innerWidth} = window;

        return [
          Math.floor(innerWidth * devicePixelRatio),
          Math.floor(innerHeight * devicePixelRatio)
        ];
    }""",
        target=ContextTarget(context["context"]),
        await_promise=False)
    return tuple(item["value"] for item in result["value"])
