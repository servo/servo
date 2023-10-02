# META: timeout=long

import pytest
import pytest_asyncio

from webdriver.bidi.modules.input import Actions
from webdriver.bidi.error import InvalidArgumentException


pytestmark = pytest.mark.asyncio


MAX_INT = 9007199254740991
MIN_INT = -MAX_INT


@pytest_asyncio.fixture
async def perform_actions(bidi_session, top_context):
    async def perform_actions(actions, context=top_context["context"]):
        return await bidi_session.input.perform_actions(
            actions=actions, context=context
        )

    yield perform_actions


def create_key_action(key_action, overrides=None, removals=None):
    action = {
        "type": key_action,
        "value": "",
    }

    if overrides is not None:
        action.update(overrides)

    if removals is not None:
        for removal in removals:
            del action[removal]

    return action


def create_pointer_action(pointer_action, overrides=None, removals=None):
    action = {
        "type": pointer_action,
        "width": 0,
        "height": 0,
        "pressure": 0.0,
        "tangentialPressure": 0.0,
        "twist": 0,
        "tiltX": 0,
        "tiltY": 0,
    }

    if pointer_action == "pointerMove":
        action.update({"x": 0, "y": 0})
    elif pointer_action in ["pointerDown", "pointerUp"]:
        action.update({"button": 0})

    if overrides is not None:
        action.update(overrides)

    if removals is not None:
        for removal in removals:
            del action[removal]

    return action


def create_wheel_action(wheel_action, overrides=None, removals=None):
    action = {
        "type": wheel_action,
        "x": 0,
        "y": 0,
        "deltaX": 0,
        "deltaY": 0,
        "deltaZ": 0,
        "deltaMode": 0,
        "origin": "viewport",
    }

    if overrides is not None:
        action.update(overrides)

    if removals is not None:
        for removal in removals:
            del action[removal]

    return action


@pytest.mark.parametrize("value", [None, True, 42, {}, []])
async def test_params_context_invalid_type(perform_actions, value):
    actions = Actions()
    actions.add_key()

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions, context=value)


@pytest.mark.parametrize("value", [None, "foo", True, 42, {}])
async def test_params_input_source_actions_invalid_type(perform_actions, value):
    with pytest.raises(InvalidArgumentException):
        await perform_actions(value)


@pytest.mark.parametrize("value", [None, "foo", True, 42, {}])
async def test_params_input_source_action_sequence_invalid_type(perform_actions, value):
    with pytest.raises(InvalidArgumentException):
        await perform_actions([value])


async def test_params_input_source_action_sequence_type_missing(perform_actions):
    actions = [
        {
            "id": "foo",
            "actions": [],
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
async def test_params_input_source_action_sequence_id_missing(
    perform_actions, action_type
):
    actions = [
        {
            "type": action_type,
            "actions": [],
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
async def test_params_input_source_action_sequence_actions_missing(
    perform_actions, action_type
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_input_source_action_sequence_type_invalid_type(
    perform_actions, value
):
    actions = [
        {
            "type": value,
            "id": "foo",
            "actions": [],
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["", "nones", "keys", "pointers", "wheels"])
async def test_params_input_source_action_sequence_type_invalid_value(
    perform_actions, action_type
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [],
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_input_source_action_sequence_id_invalid_type(
    perform_actions, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": value,
            "actions": [],
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, "foo", True, 42, {}])
async def test_params_input_source_action_sequence_actions_invalid_type(
    perform_actions, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": value,
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, "foo", True, 42, {}])
async def test_params_input_source_action_sequence_actions_actions_invalid_type(
    perform_actions, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [value],
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("value", [None, "foo", True, 42, []])
async def test_params_input_source_action_sequence_pointer_parameters_invalid_type(
    perform_actions, value
):
    actions = [{"type": "pointer", "id": "foo", "actions": [], "parameters": value}]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_input_source_action_sequence_pointer_parameters_pointer_type_invalid_type(
    perform_actions, value
):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [],
            "parameters": {
                "pointerType": value,
            },
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("value", ["", "mouses", "pens", "touchs"])
async def test_params_input_source_action_sequence_pointer_parameters_pointer_type_invalid_value(
    perform_actions, value
):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [],
            "parameters": {
                "pointerType": value,
            },
        }
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions(actions)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_input_source_action_sequence_actions_type_invalid_type(
    perform_actions, action_type, value
):
    action = {"type": value, "duration": 0}

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": action_type, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", ["", "pauses"])
async def test_params_input_source_action_sequence_actions_subtype_invalid_value(
    perform_actions, action_type, value
):
    action = {"type": value, "duration": 0}

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": action_type, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_input_source_action_sequence_actions_pause_duration_invalid_type(
    perform_actions, action_type, value
):
    action = {"type": "pause", "duration": value}

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": action_type, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [-1, MAX_INT + 1])
async def test_params_input_source_action_sequence_actions_pause_duration_invalid_value(
    perform_actions, action_type, value
):
    action = {"type": "pause", "duration": value}

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": action_type, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("value", ["", "pauses"])
async def test_params_null_action_type_invalid_value(perform_actions, value):
    action = {"type": value, "duration": 0}

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "none", "id": "foo", "actions": [action]}])


async def test_params_key_action_subtype_missing(perform_actions):
    action = create_key_action("keyDown", {"value": "f"}, removals=["type"])

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "key", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("value", ["", "keyDowns", "keyUps"])
async def test_params_key_action_subtype_invalid_value(perform_actions, value):
    action = create_key_action(value, {"value": "f"})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "key", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("key_action", ["keyDown", "keyUp"])
async def test_params_key_action_value_missing(perform_actions, key_action):
    action = create_key_action(key_action, {"value": "f"}, removals=["value"])

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "key", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("key_action", ["keyDown", "keyUp"])
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_key_action_value_invalid_type(perform_actions, key_action, value):
    action = create_key_action(key_action, {"value": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "key", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize(
    "value",
    ["fa", "\u0BA8\u0BBFb", "\u0BA8\u0BBF\u0BA8", "\u1100\u1161\u11A8c"],
)
async def test_params_key_action_value_invalid_multiple_codepoints(perform_actions, value):
    actions = [
        create_key_action("keyDown", {"value": value}),
        create_key_action("keyUp", {"value": value}),
    ]

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "key", "id": "foo", "actions": actions}])


@pytest.mark.parametrize("value", ["", "pointerDowns", "pointerMoves", "pointerUps"])
async def test_params_pointer_action_subtype_invalid_value(perform_actions, value):
    if value == "pointerMoves":
        action = create_pointer_action(value, {"x": 0, "y": 0})
    else:
        action = create_pointer_action(value, {"button": 0})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("coordinate", ["x", "y"])
async def test_params_pointer_action_up_down_button_missing(perform_actions,  coordinate):
    action = create_pointer_action("pointerMove", removals=[coordinate])

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_pointer_action_move_coordinate_invalid_type(
    perform_actions, coordinate, value
):
    action = create_pointer_action(
        "pointerMove",
        {
            "x": value if coordinate == "x" else 0,
            "y": value if coordinate == "y" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [MIN_INT - 1, MAX_INT + 1])
async def test_params_pointer_action_move_coordinate_invalid_value(
    perform_actions, coordinate, value
):
    action = create_pointer_action(
        "pointerMove",
        {
            "x": value if coordinate == "x" else 0,
            "y": value if coordinate == "y" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_pointer_action_move_origin_invalid_type(perform_actions, value):
    action = create_pointer_action("pointerMove", {"origin": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("value", ["", "pointers", "viewports"])
async def test_params_pointer_action_move_origin_invalid_value(perform_actions, value):
    action = create_pointer_action("pointerMove", {"origin": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerUp"])
async def test_params_pointer_action_up_down_button_missing(perform_actions,  pointer_action):
    action = create_pointer_action(pointer_action, removals=["button"])

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerUp"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_pointer_action_up_down_button_invalid_type(
    perform_actions, pointer_action, value
):
    action = create_pointer_action(pointer_action, {"button": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerUp"])
@pytest.mark.parametrize("value", [-1, MAX_INT + 1])
async def test_params_pointer_action_up_down_button_invalid_value(
    perform_actions, pointer_action, value
):
    action = create_pointer_action(pointer_action, {"button": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("dimension", ["width", "height"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_pointer_action_common_properties_dimensions_invalid_type(
    perform_actions, dimension, pointer_action, value
):
    action = create_pointer_action(
        pointer_action,
        {
            "width": value if dimension == "width" else 0,
            "height": value if dimension == "height" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("dimension", ["width", "height"])
@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("value", [-1, MAX_INT + 1])
async def test_params_pointer_action_common_properties_dimensions_invalid_value(
    perform_actions, dimension, pointer_action, value
):
    action = create_pointer_action(
        pointer_action,
        {
            "width": value if dimension == "width" else 0,
            "height": value if dimension == "height" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("pressure", ["pressure", "tangentialPressure"])
@pytest.mark.parametrize("value", [None, "foo", True, [], {}])
async def test_params_pointer_action_common_properties_pressure_invalid_type(
    perform_actions, pointer_action, pressure, value
):
    action = create_pointer_action(
        pointer_action,
        {
            "pressure": value if pressure == "pressure" else 0.0,
            "tangentialPressure": value if pressure == "tangentialPressure" else 0.0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_pointer_action_common_properties_twist_invalid_type(
    perform_actions, pointer_action, value
):
    action = create_pointer_action(pointer_action, {"twist": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("value", [-1, 360])
async def test_params_pointer_action_common_properties_twist_invalid_value(
    perform_actions, pointer_action, value
):
    action = create_pointer_action(pointer_action, {"twist": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("angle", ["altitudeAngle", "azimuthAngle"])
@pytest.mark.parametrize("value", [None, "foo", True, [], {}])
async def test_params_pointer_action_common_properties_angle_invalid_type(
    perform_actions, pointer_action, angle, value
):
    action = create_pointer_action(
        pointer_action,
        {
            "altitudeAngle": value if angle == "altitudeAngle" else 0.0,
            "azimuthAngle": value if angle == "azimuthAngle" else 0.0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("tilt", ["tiltX", "tiltY"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_pointer_action_common_properties_tilt_invalid_type(
    perform_actions, pointer_action, tilt, value
):
    action = create_pointer_action(
        pointer_action,
        {
            "tiltX": value if tilt == "tiltX" else 0,
            "tiltY": value if tilt == "tiltY" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("tilt", ["tiltX", "tiltY"])
@pytest.mark.parametrize("value", [-91, 91])
async def test_params_pointer_action_common_properties_tilt_invalid_value(
    perform_actions, pointer_action, tilt, value
):
    action = create_pointer_action(
        pointer_action,
        {
            "tiltX": value if tilt == "tiltX" else 0,
            "tiltY": value if tilt == "tiltY" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "pointer", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_wheel_action_scroll_coordinate_invalid_type(
    perform_actions, coordinate, value
):
    action = create_wheel_action(
        "scroll",
        {
            "x": value if coordinate == "x" else 0,
            "y": value if coordinate == "y" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [MIN_INT - 1, MAX_INT + 1])
async def test_params_wheel_action_scroll_coordinate_invalid_value(
    perform_actions, coordinate, value
):
    action = create_wheel_action(
        "scroll",
        {
            "x": value if coordinate == "x" else 0,
            "y": value if coordinate == "y" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("delta", ["x", "y"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
async def test_params_wheel_action_scroll_delta_invalid_type(
    perform_actions, delta, value
):
    action = create_wheel_action(
        "scroll",
        {
            "deltaX": value if delta == "x" else 0,
            "deltaY": value if delta == "y" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("delta", ["x", "y"])
@pytest.mark.parametrize("value", [MIN_INT - 1, MAX_INT + 1])
async def test_params_wheel_action_scroll_delta_invalid_value(
    perform_actions, delta, value
):
    action = create_wheel_action(
        "scroll",
        {
            "deltaX": value if delta == "x" else 0,
            "deltaY": value if delta == "y" else 0,
        },
    )

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
async def test_params_wheel_action_scroll_origin_invalid_type(perform_actions, value):
    action = create_wheel_action("scroll", {"origin": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("value", ["", "pointers", "viewports"])
async def test_params_wheel_action_scroll_origin_invalid_value(perform_actions, value):
    action = create_wheel_action("scroll", {"origin": value})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


async def test_params_wheel_action_scroll_origin_pointer_not_supported(perform_actions):
    # Pointer origin isn't currently supported for wheel input source
    # See: https://github.com/w3c/webdriver/issues/1758
    action = create_wheel_action("scroll", {"origin": "pointer"})

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("missing", ["x", "y", "deltaX", "deltaY"])
async def test_params_wheel_action_scroll_property_missing(perform_actions, missing):
    action = create_wheel_action("scroll", removals=[missing])

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": "wheel", "id": "foo", "actions": [action]}])


# Element origin tests for pointer and wheel input sources

@pytest.mark.parametrize("input_source", ["pointer", "wheel"])
@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_origin_element_type_invalid_type(
    perform_actions, input_source, value
):
    origin = {"origin": {"type": value}}

    if input_source == "pointer":
        action = create_pointer_action("pointerMove", origin)
    elif input_source == "wheel":
        action = create_wheel_action("scroll", origin)

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": input_source, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("input_source", ["pointer", "wheel"])
async def test_params_origin_element_element_missing(
    perform_actions, input_source
):
    origin = {"origin": {"type": "element"}}

    if input_source == "pointer":
        action = create_pointer_action("pointerMove", origin)
    elif input_source == "wheel":
        action = create_wheel_action("scroll", origin)

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": input_source, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("input_source", ["pointer", "wheel"])
@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_origin_element_element_invalid_type(
    perform_actions, input_source, value
):
    origin = {"origin": {"type": "element", "element": value}}

    if input_source == "pointer":
        action = create_pointer_action("pointerMove", origin)
    elif input_source == "wheel":
        action = create_wheel_action("scroll", origin)

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": input_source, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("input_source", ["pointer", "wheel"])
async def test_params_origin_element_element_sharedid_missing(
    perform_actions, input_source
):
    origin = {"origin": {"type": "element", "element": {}}}

    if input_source == "pointer":
        action = create_pointer_action("pointerMove", origin)
    elif input_source == "wheel":
        action = create_wheel_action("scroll", origin)

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": input_source, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("input_source", ["pointer", "wheel"])
@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_origin_element_element_sharedid_invalid_type(
    perform_actions, input_source, value
):
    origin = {"origin": {"type": "element", "element": {"sharedId": value}}}

    if input_source == "pointer":
        action = create_pointer_action("pointerMove", origin)
    elif input_source == "wheel":
        action = create_wheel_action("scroll", origin)

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": input_source, "id": "foo", "actions": [action]}])


@pytest.mark.parametrize("input_source", ["pointer", "wheel"])
async def test_params_origin_element_invalid_with_shared_reference(
    bidi_session, top_context, get_actions_origin_page, get_element, perform_actions, input_source
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_actions_origin_page(""),
        wait="complete",
    )

    origin = {"origin": await get_element("#inner")}

    if input_source == "pointer":
        action = create_pointer_action("pointerMove", origin)
    elif input_source == "wheel":
        action = create_wheel_action("scroll", origin)

    with pytest.raises(InvalidArgumentException):
        await perform_actions([{"type": input_source, "id": "foo", "actions": [action]}])
