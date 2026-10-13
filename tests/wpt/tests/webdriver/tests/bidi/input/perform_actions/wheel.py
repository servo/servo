import pytest

from webdriver.bidi.error import MoveTargetOutOfBoundsException, NoSuchFrameException
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from . import assert_scroll_position

pytestmark = pytest.mark.asyncio


async def test_invalid_browsing_context(bidi_session):
    actions = Actions()
    actions.add_wheel()

    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(actions=actions, context="foo")


@pytest.mark.parametrize("origin", ["element", "viewport"])
async def test_params_actions_origin_outside_viewport(
    bidi_session, top_context, test_actions_wheel_page, get_element, origin
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(),
        wait="complete",
    )

    if origin == "element":
        element = await get_element("#not-scrollable")
        origin = get_element_origin(element)

    actions = Actions()
    actions.add_wheel().scroll(x=-100, y=-100, delta_x=10, delta_y=20, origin=origin)

    with pytest.raises(MoveTargetOutOfBoundsException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


@pytest.mark.parametrize(
    "delta_x, delta_y",
    [(50, 0), (0, 60), (70, 80)],
    ids=["delta-x", "delta-y", "delta-x-and-y"],
)
async def test_scroll_direction(
    bidi_session, top_context, test_actions_wheel_page, get_element, delta_x, delta_y
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(),
        wait="complete",
    )

    target = await get_element("#scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_x, delta_y=delta_y,
        origin=get_element_origin(target),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    await assert_scroll_position(bidi_session, top_context, target, delta_x, delta_y)


@pytest.mark.parametrize("mode", ["open", "closed"])
async def test_scroll_element_in_shadow_tree(
    bidi_session, new_tab, test_actions_wheel_page, mode
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_actions_wheel_page(shadow=mode),
        wait="complete",
    )

    custom_element = await bidi_session.script.evaluate(
        expression='document.querySelector("#custom-element")',
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )
    shadow_root = custom_element["value"]["shadowRoot"]

    scrollable = await bidi_session.script.call_function(
        function_declaration='sr => sr.querySelector("#shadow-scrollable")',
        target=ContextTarget(new_tab["context"]),
        arguments=[shadow_root],
        await_promise=False,
    )

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=55, delta_y=75,
        origin=get_element_origin(scrollable),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=new_tab["context"]
    )

    await assert_scroll_position(bidi_session, new_tab, scrollable, 55, 75)


@pytest.mark.parametrize("scale", ["0.5", "1.0", "1.5"])
async def test_scroll_position_for_scaled_layout_viewport(
    bidi_session, new_tab, inline, scale
):
    url = inline(f"""
        <meta name="viewport" content="width=device-width,initial-scale={scale}">
        <div id="scroller" style="overflow: auto; width: 250px; height: 150px">
          <iframe srcdoc="foo" style="width: 200px; height: 100px"></iframe>
          <div style="height: 2000px; width: 2000px"></div>
        </div>
    """)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete",
    )

    iframes = await bidi_session.browsing_context.locate_nodes(
        context=new_tab["context"], locator={"type": "css", "value": "iframe"}
    )

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=60, delta_y=80,
        origin=get_element_origin(iframes[0]),
        duration=100,
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=new_tab["context"]
    )

    scrollers = await bidi_session.browsing_context.locate_nodes(
        context=new_tab["context"],
        locator={"type": "css", "value": "#scroller"},
    )

    await assert_scroll_position(
        bidi_session, new_tab, scrollers[0], 60, 80
    )
