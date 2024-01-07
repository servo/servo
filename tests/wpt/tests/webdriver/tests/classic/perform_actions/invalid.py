import pytest

from webdriver.error import InvalidArgumentException

from tests.support.asserts import assert_error
from . import perform_actions


MAX_INT = 9007199254740991
MIN_INT = -MAX_INT


def create_pointer_common_object(pointer_action, overrides):
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
    else:
        action.update({"button": 0})

    action.update(overrides)

    return action


@pytest.mark.parametrize("value", [None, "foo", True, 42, {}])
def test_input_source_action_sequence_invalid_type(session, value):
    response = perform_actions(session, value)
    assert_error(response, "invalid argument")


def test_input_source_action_sequence_missing_type(session):
    actions = [
        {
            "id": "foo",
            "actions": [],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
def test_input_source_action_sequence_missing_id(session, action_type):
    actions = [
        {
            "type": action_type,
            "actions": [],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
def test_input_source_action_sequence_missing_actions(session, action_type):
    actions = [
        {
            "type": action_type,
            "id": "foo",
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_input_source_action_sequence_type_invalid_type(session, value):
    actions = [
        {
            "type": value,
            "id": "foo",
            "actions": [],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


def test_input_source_action_sequence_type_invalid_value(session):
    for invalid_value in ["", "nones", "keys", "pointers", "wheels"]:
        actions = [
            {
                "type": invalid_value,
                "id": "foo",
                "actions": [],
            }
        ]
        response = perform_actions(session, actions)
        assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_input_source_action_sequence_id_invalid_type(session, action_type, value):
    actions = [
        {
            "type": action_type,
            "id": value,
            "actions": [],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, "foo", True, 42, {}])
def test_input_source_action_sequence_actions_invalid_type(session, action_type, value):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": value,
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, "foo", True, 42, []])
def test_input_source_action_sequence_pointer_parameters_invalid_type(session, value):
    actions = [{"type": "pointer", "id": "foo", "actions": [], "parameters": value}]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_input_source_action_sequence_pointer_parameters_pointer_type_invalid_type(
    session, value
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
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", ["", "mouses", "pens", "touchs"])
def test_input_source_action_sequence_pointer_parameters_pointer_type_invalid_value(
    session, value
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
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_input_source_action_sequence_actions_type_invalid_type(
    session, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [
                {
                    "type": value,
                    "duration": 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", ["", "pauses"])
def test_input_source_action_sequence_actions_subtype_invalid_value(
    session, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [
                {
                    "type": value,
                    "duration": 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_input_source_action_sequence_actions_pause_duration_invalid_type(
    session, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [
                {
                    "type": "pause",
                    "duration": value,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
@pytest.mark.parametrize("value", [-1, MAX_INT + 1])
def test_input_source_action_sequence_actions_pause_duration_invalid_value(
    session, action_type, value
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [{"type": "pause", "duration": value}],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", ["", "pauses"])
def test_null_action_type_invalid_value(session, value):
    actions = [
        {
            "type": "none",
            "id": "foo",
            "actions": [
                {
                    "type": value,
                    "duration": 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", ["", "keyDowns", "keyUps"])
def test_key_action_subtype_invalid_value(session, value):
    actions = [
        {
            "type": "key",
            "id": "foo",
            "actions": [
                {
                    "type": value,
                    "value": "f",
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("key_action", ["keyDown", "keyUp"])
@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_key_action_value_invalid_type(session, key_action, value):
    actions = [
        {
            "type": "key",
            "id": "foo",
            "actions": [
                {
                    "type": key_action,
                    "value": value,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", ["", "pointerDowns", "pointerMoves", "pointerUps"])
def test_pointer_action_subtype_invalid_value(session, value):
    if value == "pointerMoves":
        actions = [
            {
                "type": "pointer",
                "id": "foo",
                "actions": [
                    {
                        "type": "pointerMoves",
                        "x": 0,
                        "y": 0,
                    }
                ],
            }
        ]
    else:
        actions = [
            {
                "type": "pointer",
                "id": "foo",
                "actions": [
                    {
                        "type": value,
                        "button": 0,
                    }
                ],
            }
        ]

    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_pointer_action_move_coordinate_invalid_type(session, coordinate, value):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [
                {
                    "type": "pointerMove",
                    "x": value if coordinate == "x" else 0,
                    "y": value if coordinate == "y" else 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [MIN_INT - 1, MAX_INT + 1])
def test_pointer_action_move_coordinate_invalid_value(session, coordinate, value):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [
                {
                    "type": "pointerMove",
                    "x": value if coordinate == "x" else 0,
                    "y": value if coordinate == "y" else 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_pointer_action_move_origin_invalid_type(session, value):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [{"type": "pointerMove", "x": 0, "y": 0, "origin": value}],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", ["", "pointers", "viewports"])
def test_pointer_action_move_origin_invalid_value(session, value):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [{"type": "pointerMove", "x": 0, "y": 0, "origin": value}],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize(
    "value",
    [
        {"frame-075b-4da1-b6ba-e579c2d3230a": "foo"},
        {"shadow-6066-11e4-a52e-4f735466cecf": "foo"},
        {"window-fcc6-11e5-b4f8-330a88ab9d7f": "foo"},
    ],
    ids=["frame", "shadow", "window"],
)
def test_pointer_action_move_origin_element_invalid_type(session, value):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [{"type": "pointerMove", "x": 0, "y": 0, "origin": value}],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


def test_pointer_action_move_origin_element_invalid_value(session):
    value = {"element-6066-11e4-a52e-4f735466cecf": "foo"}

    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [{"type": "pointerMove", "x": 0, "y": 0, "origin": value}],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "no such element")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerUp"])
def test_pointer_action_up_down_button_missing(session, pointer_action):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "actions": [
                {
                    "type": pointer_action,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerUp"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_pointer_action_up_down_button_invalid_type(session, pointer_action, value):
    action = create_pointer_common_object(pointer_action, {"button": value})

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerUp"])
@pytest.mark.parametrize("value", [-1, MAX_INT + 1])
def test_pointer_action_up_down_button_invalid_value(session, pointer_action, value):
    action = create_pointer_common_object(pointer_action, {"button": value})

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("dimension", ["width", "height"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_pointer_action_common_properties_dimensions_invalid_type(
    session, dimension, pointer_action, value
):
    action = create_pointer_common_object(
        pointer_action,
        {
            "width": value if dimension == "width" else 0,
            "height": value if dimension == "height" else 0,
        },
    )

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("dimension", ["width", "height"])
@pytest.mark.parametrize("value", [-1, MAX_INT + 1])
def test_pointer_action_common_properties_dimensions_invalid_value(
    session, dimension, pointer_action, value
):
    action = create_pointer_common_object(
        pointer_action,
        {
            "width": value if dimension == "width" else 0,
            "height": value if dimension == "height" else 0,
        },
    )

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("pressure", ["pressure", "tangentialPressure"])
@pytest.mark.parametrize("value", [None, "foo", True, [], {}])
def test_pointer_action_common_properties_pressure_invalid_type(
    session, pointer_action, pressure, value
):
    action = create_pointer_common_object(
        pointer_action,
        {
            "pressure": value if pressure == "pressure" else 0.0,
            "tangentialPressure": value if pressure == "tangentialPressure" else 0.0,
        },
    )

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_pointer_action_common_properties_twist_invalid_type(
    session, pointer_action, value
):
    action = create_pointer_common_object(pointer_action, {"twist": value})

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("value", [-1, 360])
def test_pointer_action_common_properties_twist_invalid_value(
    session, pointer_action, value
):
    action = create_pointer_common_object(pointer_action, {"twist": value})

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("angle", ["altitudeAngle", "azimuthAngle"])
@pytest.mark.parametrize("value", [None, "foo", True, [], {}])
def test_pointer_action_common_properties_angle_invalid_type(
    session, pointer_action, angle, value
):
    action = create_pointer_common_object(
        pointer_action,
        {
            "altitudeAngle": value if angle == "altitudeAngle" else 0.0,
            "azimuthAngle": value if angle == "azimuthAngle" else 0.0,
        },
    )

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("tilt", ["tiltX", "tiltY"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_pointer_action_common_properties_tilt_invalid_type(
    session, pointer_action, tilt, value
):
    action = create_pointer_common_object(
        pointer_action,
        {
            "tiltX": value if tilt == "tiltX" else 0,
            "tiltY": value if tilt == "tiltY" else 0,
        },
    )

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("pointer_action", ["pointerDown", "pointerMove", "pointerUp"])
@pytest.mark.parametrize("tilt", ["tiltX", "tiltY"])
@pytest.mark.parametrize("value", [-91, 91])
def test_pointer_action_common_properties_tilt_invalid_value(
    session, pointer_action, tilt, value
):
    action = create_pointer_common_object(
        pointer_action,
        {
            "tiltX": value if tilt == "tiltX" else 0,
            "tiltY": value if tilt == "tiltY" else 0,
        },
    )

    response = perform_actions(
        session, [{"type": "pointer", "id": "foo", "actions": [action]}]
    )
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_wheel_action_scroll_coordinate_invalid_type(session, coordinate, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": value if coordinate == "x" else 0,
                    "y": value if coordinate == "y" else 0,
                    "deltaX": 0,
                    "deltaY": 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("coordinate", ["x", "y"])
@pytest.mark.parametrize("value", [MIN_INT - 1, MAX_INT + 1])
def test_wheel_action_scroll_coordinate_invalid_value(session, coordinate, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": value if coordinate == "x" else 0,
                    "y": value if coordinate == "y" else 0,
                    "deltaX": 0,
                    "deltaY": 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("delta", ["x", "y"])
@pytest.mark.parametrize("value", [None, "foo", True, 0.1, [], {}])
def test_wheel_action_scroll_delta_invalid_type(session, delta, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": 0,
                    "y": 0,
                    "deltaX": value if delta == "x" else 0,
                    "deltaY": value if delta == "y" else 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("delta", ["x", "y"])
@pytest.mark.parametrize("value", [MIN_INT - 1, MAX_INT + 1])
def test_wheel_action_scroll_delta_invalid_value(session, delta, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "deltaX": value if delta == "x" else 0,
                    "deltaY": value if delta == "y" else 0,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [None, True, 42, [], {}])
def test_wheel_action_scroll_origin_invalid_type(session, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": 0,
                    "y": 0,
                    "deltaX": 0,
                    "deltaY": 0,
                    "origin": value,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", ["", "pointers", "viewports"])
def test_wheel_action_scroll_origin_invalid_value(session, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": 0,
                    "y": 0,
                    "deltaX": 0,
                    "deltaY": 0,
                    "origin": value,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


def test_wheel_action_scroll_origin_pointer_not_supported(session):
    # Pointer origin isn't currently supported for wheel input source
    # See: https://github.com/w3c/webdriver/issues/1758

    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": 0,
                    "y": 0,
                    "deltaX": 0,
                    "deltaY": 0,
                    "origin": "pointer",
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize(
    "value",
    [
        {"frame-075b-4da1-b6ba-e579c2d3230a": "foo"},
        {"shadow-6066-11e4-a52e-4f735466cecf": "foo"},
        {"window-fcc6-11e5-b4f8-330a88ab9d7f": "foo"},
    ],
    ids=["frame", "shadow", "window"],
)
def test_wheel_action_scroll_origin_element_invalid_type(session, value):
    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": 0,
                    "y": 0,
                    "deltaX": 0,
                    "deltaY": 0,
                    "origin": value,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "invalid argument")


def test_wheel_action_scroll_origin_element_invalid_value(session):
    value = {"element-6066-11e4-a52e-4f735466cecf": "foo"}

    actions = [
        {
            "type": "wheel",
            "id": "foo",
            "actions": [
                {
                    "type": "scroll",
                    "x": 0,
                    "y": 0,
                    "deltaX": 0,
                    "deltaY": 0,
                    "origin": value,
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_error(response, "no such element")


@pytest.mark.parametrize("missing", ["x", "y", "deltaX", "deltaY"])
def test_wheel_action_scroll_missing_property(
    session, test_actions_scroll_page, wheel_chain, missing
):
    actions = wheel_chain.scroll(0, 0, 5, 10, origin="viewport")
    del actions._actions[-1][missing]

    with pytest.raises(InvalidArgumentException):
        actions.perform()
