from math import floor
from ... import (
    get_device_pixel_ratio,
    get_document_dimensions,
    get_element_dimensions,
    get_viewport_dimensions,
    remote_mapping_to_dict,
)

from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.modules.browsing_context import ElementOptions


async def get_element_coordinates(bidi_session, context, element):
    """Get the coordinates of the element.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :param element: Serialized element
    :returns: Tuple of (int, int) containing element x, element y coordinates.
    """
    result = await bidi_session.script.call_function(
        arguments=[element],
        function_declaration="""(element) => {
            const rect = element.getBoundingClientRect();
            return { x: rect.x, y: rect.y }
        }""",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    value = remote_mapping_to_dict(result["value"])

    return (value["x"], value["y"])


async def get_page_y_offset(bidi_session, context):
    """Get the window.pageYOffset of the context's viewport.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :returns: int value of window.pageYOffset.
    """
    result = await bidi_session.script.evaluate(
        expression="window.pageYOffset",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    return result["value"]


async def get_physical_element_dimensions(bidi_session, context, element):
    """Get the physical dimensions of the element.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :param element: Serialized element
    :returns: Tuple of (int, int) containing element width, element height.
    """
    element_dimensions = await get_element_dimensions(bidi_session, context, element)
    dpr = await get_device_pixel_ratio(bidi_session, context)
    return (floor(element_dimensions["width"] * dpr), floor(element_dimensions["height"] * dpr))


async def get_physical_viewport_dimensions(bidi_session, context):
    """Get the physical dimensions of the context's viewport.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :returns: Tuple of (int, int) containing viewport width, viewport height.
    """
    viewport = await get_viewport_dimensions(bidi_session, context)
    dpr = await get_device_pixel_ratio(bidi_session, context)
    return (floor(viewport["width"] * dpr), floor(viewport["height"] * dpr))


async def get_physical_document_dimensions(bidi_session, context):
    """Get the physical dimensions of the context's document.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :returns: Tuple of (int, int) containing document width, document height.
    """
    document = await get_document_dimensions(bidi_session, context)
    dpr = await get_device_pixel_ratio(bidi_session, context)
    return (floor(document["width"] * dpr), floor(document["height"] * dpr))


async def get_reference_screenshot(bidi_session, inline, context, html):
    """Get the reference screenshot for the given context and html.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :param html: Html string
    :returns: Screenshot image.
    """
    url = inline(html)
    await bidi_session.browsing_context.navigate(
        context=context, url=url, wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(context),
    )

    return await bidi_session.browsing_context.capture_screenshot(
        context=context,
        clip=ElementOptions(element=element),
    )
