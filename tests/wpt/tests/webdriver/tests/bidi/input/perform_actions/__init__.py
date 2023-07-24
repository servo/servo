from webdriver.bidi.modules.script import ContextTarget

from ... import get_viewport_dimensions, remote_mapping_to_dict


async def get_inview_center_bidi(bidi_session, context, element):
    elem_rect = await get_element_rect(bidi_session,
                                       context=context,
                                       element=element)
    viewport_rect = await get_viewport_dimensions(bidi_session,
                                                  context=context)

    x = {
        "left": max(0, min(elem_rect["x"],
                           elem_rect["x"] + elem_rect["width"])),
        "right": min(
            viewport_rect["width"],
            max(elem_rect["x"], elem_rect["x"] + elem_rect["width"]),
        ),
    }

    y = {
        "top": max(0, min(elem_rect["y"],
                          elem_rect["y"] + elem_rect["height"])),
        "bottom": min(
            viewport_rect["height"],
            max(elem_rect["y"], elem_rect["y"] + elem_rect["height"]),
        ),
    }

    return {
        "x": (x["left"] + x["right"]) / 2,
        "y": (y["top"] + y["bottom"]) / 2,
    }


async def get_element_rect(bidi_session, context, element):
    result = await bidi_session.script.call_function(
        function_declaration="""
el => el.getBoundingClientRect().toJSON()
""",
        arguments=[element],
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    return remote_mapping_to_dict(result["value"])
