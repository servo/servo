import pytest

from tests.support.sync import AsyncPoll
from webdriver.bidi.error import MoveTargetOutOfBoundsException, NoSuchFrameException
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_invalid_browsing_context(bidi_session):
    actions = Actions()
    actions.add_wheel()

    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(actions=actions, context="foo")


@pytest.mark.parametrize("origin", ["element", "viewport"])
async def test_params_actions_origin_outside_viewport(
    bidi_session, setup_wheel_test, top_context, get_element, origin
):
    if origin == "element":
        element = await get_element("#not-scrollable")
        origin = get_element_origin(element)

    actions = Actions()
    actions.add_wheel().scroll(x=-100, y=-100, delta_x=10, delta_y=20, origin=origin)

    with pytest.raises(MoveTargetOutOfBoundsException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


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
        x=0,
        y=0,
        delta_x=20,
        delta_y=50,
        origin=get_element_origin(iframes["nodes"][0]),
        duration=100
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=new_tab["context"]
    )

    nodes = await bidi_session.browsing_context.locate_nodes(
        context=new_tab["context"], locator={"type": "css", "value": "#scroller"}
    )

    async def assert_scroll_position(_):
        result = await bidi_session.script.call_function(
            function_declaration="scroller => [scroller.scrollLeft, scroller.scrollTop]",
            target=ContextTarget(new_tab["context"]),
            arguments=[nodes["nodes"][0]],
            await_promise=False,
        )
        scroll_left = result["value"][0]["value"]
        scroll_top = result["value"][1]["value"]
        assert scroll_left == pytest.approx(
            20, abs=1.0
        ), "Did not reach scrollLeft position"
        assert scroll_top == pytest.approx(
            50, abs=1.0
        ), "Did not reach scrollTop position"

    wait = AsyncPoll(bidi_session, timeout=0.5)
    await wait.until(assert_scroll_position)
