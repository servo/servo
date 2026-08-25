import pytest

from tests.support.classic.asserts import assert_success
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


def test_interspersed_wheel_pointermove(session, wheel_chain, mouse_chain, inline):
    session.url = inline("""
        <div id='target' style='width: 200px; height: 200px; background: red; overflow: scroll;'>
            <div style='height: 1000px;'></div>
        </div>
        <script>
            window.events = [];
            window.onwheel = () => window.events.push('wheel');
            window.onmousemove = () => window.events.push('move');
        </script>
    """)

    target = session.find.css("#target", all=False)

    wheel_chain.scroll(0, 0, 0, 150, duration=500, origin=target)
    mouse_chain.pointer_move(80, 80, duration=500, origin=target)

    session.actions.perform([wheel_chain.dict, mouse_chain.dict])

    events = session.execute_script("return window.events")

    assert "wheel" in events
    assert "move" in events

    first_move = events.index("move")
    last_wheel = len(events) - 1 - events[::-1].index("wheel")

    assert first_move < last_wheel, f"Events were not interspersed: {events}"
