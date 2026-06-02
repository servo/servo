import pytest

from tests.support.asserts import assert_success
from . import perform_actions


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
def test_input_source_action_sequence_actions_pause_duration_valid(
    session, action_type
):
    for valid_duration in [0, 1]:
        actions = [
            {
                "type": action_type,
                "id": "foo",
                "actions": [{"type": "pause", "duration": valid_duration}],
            }
        ]
        response = perform_actions(session, actions)
        assert_success(response)


@pytest.mark.parametrize("action_type", ["none", "key", "pointer", "wheel"])
def test_input_source_action_sequence_actions_pause_duration_missing(
    session, action_type
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [
                {
                    "type": "pause",
                }
            ],
        }
    ]
    response = perform_actions(session, actions)
    assert_success(response)


@pytest.mark.parametrize("action_type", ["none", "key", "wheel"])
def test_input_source_action_sequence_pointer_parameters_not_processed(
    session, action_type
):
    actions = [
        {
            "type": action_type,
            "id": "foo",
            "actions": [],
            "parameters": True,
        }
    ]
    response = perform_actions(session, actions)
    assert_success(response)
