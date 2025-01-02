import pytest

from webdriver.bidi.error import NoSuchElementException
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from . import (
    get_inview_center_bidi,
    remote_mapping_to_dict,
)

pytestmark = pytest.mark.asyncio


async def get_click_coordinates(bidi_session, context):
    """Helper to get recorded click coordinates on a page generated with the
    actions_origins_doc fixture."""
    result = await bidi_session.script.evaluate(
        expression="window.coords",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    return remote_mapping_to_dict(result["value"])


async def test_viewport_inside(bidi_session, top_context,
                               get_actions_origin_page):
    point = {"x": 50, "y": 50}

    url = get_actions_origin_page(
        "width: 100px; height: 50px; background: green;")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    actions = Actions()
    actions.add_pointer().pointer_move(x=point["x"], y=point["y"])
    await bidi_session.input.perform_actions(actions=actions,
                                             context=top_context["context"])

    click_coords = await get_click_coordinates(bidi_session,
                                               context=top_context)
    assert click_coords["x"] == pytest.approx(point["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(point["y"], abs=1.0)


async def test_pointer_inside(bidi_session, top_context,
                              get_actions_origin_page):
    start_point = {"x": 50, "y": 50}
    offset = {"x": 10, "y": 5}

    url = get_actions_origin_page(
        "width: 100px; height: 50px; background: green;")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    actions = Actions()
    (actions.add_pointer().pointer_move(
        x=start_point["x"], y=start_point["y"]).pointer_move(x=offset["x"],
                                                             y=offset["y"],
                                                             origin="pointer"))

    await bidi_session.input.perform_actions(actions=actions,
                                             context=top_context["context"])

    click_coords = await get_click_coordinates(bidi_session,
                                               context=top_context)
    assert click_coords["x"] == pytest.approx(start_point["x"] + offset["x"],
                                              abs=1.0)
    assert click_coords["y"] == pytest.approx(start_point["y"] + offset["y"],
                                              abs=1.0)


@pytest.mark.parametrize(
    "doc",
    [
        "width: 100px; height: 50px; background: green;",
        """width: 100px; height: 50px; background: green;
           position: relative; left: -50px; top: -25px;""",
    ],
    ids=["element fully visible", "element partly visible"],
)
@pytest.mark.parametrize("offset_x, offset_y", [(10, 15), (0, 0)])
async def test_element_center_point_with_offset(
    bidi_session,
    top_context,
    get_actions_origin_page,
    get_element,
    doc,
    offset_x,
    offset_y,
):
    url = get_actions_origin_page(doc)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    elem = await get_element("#inner")
    center = await get_inview_center_bidi(bidi_session,
                                          context=top_context,
                                          element=elem)

    actions = Actions()
    actions.add_pointer().pointer_move(x=offset_x,
                                       y=offset_y,
                                       origin=get_element_origin(elem))
    await bidi_session.input.perform_actions(actions=actions,
                                             context=top_context["context"])

    click_coords = await get_click_coordinates(bidi_session,
                                               context=top_context)
    assert click_coords["x"] == pytest.approx(center["x"] + offset_x, abs=1.0)
    assert click_coords["y"] == pytest.approx(center["y"] + offset_y, abs=1.0)


async def test_element_larger_than_viewport(bidi_session, top_context,
                                            get_actions_origin_page,
                                            get_element):
    url = get_actions_origin_page(
        "width: 300vw; height: 300vh; background: green;")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    elem = await get_element("#inner")
    center = await get_inview_center_bidi(bidi_session,
                                          context=top_context,
                                          element=elem)

    actions = Actions()
    actions.add_pointer().pointer_move(x=0,
                                       y=0,
                                       origin=get_element_origin(elem))
    await bidi_session.input.perform_actions(actions=actions,
                                             context=top_context["context"])

    click_coords = await get_click_coordinates(bidi_session,
                                               context=top_context)
    assert click_coords["x"] == pytest.approx(center["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(center["y"], abs=1.0)


@pytest.mark.parametrize(
    "expression",
    [
        "document.querySelector('input#button').attributes[0]",
        "document.querySelector('#with-text-node').childNodes[0]",
        """document.createProcessingInstruction("xml-stylesheet", "href='foo.css'")""",
        "document.querySelector('#with-comment').childNodes[0]",
        "document",
        "document.doctype",
        "document.createDocumentFragment()",
        "document.querySelector('#custom-element').shadowRoot",
    ],
    ids=[
        "attribute",
        "text node",
        "processing instruction",
        "comment",
        "document",
        "doctype",
        "document fragment",
        "shadow root",
    ]
)
async def test_params_actions_origin_no_such_element(
    bidi_session, top_context, get_test_page, expression
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(),
        wait="complete",
    )

    node = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    actions = Actions()
    actions.add_pointer().pointer_move(x=0, y=0, origin=get_element_origin(node))

    with pytest.raises(NoSuchElementException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )
