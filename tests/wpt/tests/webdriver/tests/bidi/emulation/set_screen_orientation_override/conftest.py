import pytest_asyncio
import pytest

from webdriver.bidi.modules.script import ContextTarget
from . import get_angle
from ... import remote_mapping_to_dict


@pytest_asyncio.fixture
async def get_screen_orientation(bidi_session):
    async def get_screen_orientation(context):
        # Activation is required, as orientation is only available on an active
        # context.
        await bidi_session.browsing_context.activate(context=context["context"])

        result = await bidi_session.script.evaluate(
            expression="""({
                    angle: screen.orientation.angle,
                    type: screen.orientation.type
                })
            """,
            target=ContextTarget(context["context"]),
            await_promise=True,
        )

        return remote_mapping_to_dict(result["value"])

    return get_screen_orientation


@pytest_asyncio.fixture
async def default_screen_orientation(top_context, get_screen_orientation):
    return await get_screen_orientation(top_context)


def __orientations_angles_match(orientation_1, orientation_2):
    """
    Helper function for finding a unique orientation for testing purposes.
    """
    angle_1 = orientation_1["angle"] if "angle" in orientation_1 else get_angle(
        orientation_1["type"], orientation_1["natural"])
    angle_2 = orientation_2["angle"] if "angle" in orientation_2 else get_angle(
        orientation_2["type"], orientation_2["natural"])
    return angle_1 == angle_2


@pytest.fixture
def some_bidi_screen_orientation(default_screen_orientation):
    """
    Some orientation not equal to the default one.
    """
    natural_values = ["landscape", "portrait"]
    type_values = ["portrait-primary", "portrait-secondary",
                   "landscape-primary", "landscape-secondary"]
    for natural in natural_values:
        for _type in type_values:
            orientation = {
                "natural": natural,
                "type": _type
            }
            if not __orientations_angles_match(default_screen_orientation,
                                               orientation):
                # Return any orientation that has angle different from the
                # default.
                return orientation

    raise Exception(
        f"Unexpectedly could not find orientation different from the default {default_screen_orientation}")


@pytest.fixture
def some_web_screen_orientation(some_bidi_screen_orientation):
    return {
        "type": some_bidi_screen_orientation["type"],
        "angle": get_angle(some_bidi_screen_orientation["type"],
                           some_bidi_screen_orientation["natural"])
    }


@pytest.fixture
def another_bidi_screen_orientation(default_screen_orientation,
        some_bidi_screen_orientation):
    natural_values = ["landscape", "portrait"]
    type_values = ["portrait-primary", "portrait-secondary",
                   "landscape-primary", "landscape-secondary"]
    for natural in natural_values:
        for _type in type_values:
            orientation = {
                "natural": natural,
                "type": _type
            }
            if not __orientations_angles_match(default_screen_orientation,
                                               orientation) and not __orientations_angles_match(
                some_bidi_screen_orientation, orientation):
                # Return any orientation that has angle different from the
                # default and from the `some`.
                return orientation

    raise Exception(
        f"Unexpectedly could not find orientation different from the default {default_screen_orientation} and some {some_bidi_screen_orientation}")


@pytest.fixture
def another_web_screen_orientation(another_bidi_screen_orientation):
    return {
        "type": another_bidi_screen_orientation["type"],
        "angle": get_angle(another_bidi_screen_orientation["type"],
                           another_bidi_screen_orientation["natural"])
    }
