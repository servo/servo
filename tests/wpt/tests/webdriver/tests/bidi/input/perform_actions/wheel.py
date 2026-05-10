import pytest

from webdriver.bidi.error import MoveTargetOutOfBoundsException, NoSuchFrameException
from webdriver.bidi.modules.input import Actions, get_element_origin


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
