from webdriver.bidi.modules.script import ContextTarget


def remote_mapping_to_dict(js_object):
    obj = {}
    for key, value in js_object:
        obj[key] = value["value"]

    return obj


async def get_inview_center_bidi(bidi_session, context, element):
    elem_rect = await get_element_rect(bidi_session, context=context, element=element)
    viewport_rect = await get_viewport_rect(bidi_session, context=context)

    x = {
        "left": max(0, min(elem_rect["x"], elem_rect["x"] + elem_rect["width"])),
        "right": min(
            viewport_rect["width"],
            max(elem_rect["x"], elem_rect["x"] + elem_rect["width"]),
        ),
    }

    y = {
        "top": max(0, min(elem_rect["y"], elem_rect["y"] + elem_rect["height"])),
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


async def get_viewport_rect(bidi_session, context):
    expression = """
        ({
          height: window.innerHeight || document.documentElement.clientHeight,
          width: window.innerWidth || document.documentElement.clientWidth,
        });
    """
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    return remote_mapping_to_dict(result["value"])
