from webdriver.bidi.modules.script import ContextTarget

from ... import get_device_pixel_ratio, get_viewport_dimensions


async def get_physical_viewport_dimensions(bidi_session, context):
    """Get the physical dimensions of the context's viewport.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :returns: Tuple of (int, int) containing viewport width, viewport height.
    """
    viewport = await get_viewport_dimensions(bidi_session, context)
    dpr = await get_device_pixel_ratio(bidi_session, context)
    return (viewport["width"] * dpr, viewport["height"] * dpr)
