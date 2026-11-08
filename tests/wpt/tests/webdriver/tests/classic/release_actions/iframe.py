import pytest

from webdriver import WebElement
from tests.support.classic.asserts import assert_success


def perform_actions(session, actions):
    return session.transport.send(
        "POST",
        "/session/{session_id}/actions".format(session_id=session.session_id),
        {"actions": actions})


def release_actions(session):
    return session.transport.send(
        "DELETE",
        "/session/{session_id}/actions".format(**vars(session)),
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_release_actions_in_frame(session, inline, iframe, domain):
    target = "<button id='b'>go</button>"
    session.url = inline(iframe(target, domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    button = session.find.css("#b", all=False)

    # Record the events that releasing the pressed button dispatches in the frame.
    session.execute_script(
        """
        window.releaseEvents = [];
        arguments[0].addEventListener("pointerup", e => window.releaseEvents.push(e.type));
        arguments[0].addEventListener("mouseup", e => window.releaseEvents.push(e.type));
        """,
        args=(button,),
    )

    # Leave a pointer button pressed on the element inside the frame, so that
    # Release Actions has to release it there.
    # session.transport.send posts the action dicts as raw JSON, so the element has to be
    # referenced in wire format rather than passed as a WebElement.
    origin = {WebElement.identifier: button.id}
    actions = [
        {
            "type": "pointer",
            "id": "pointer_id",
            "parameters": {"pointerType": "mouse"},
            "actions": [
                {"type": "pointerMove", "x": 0, "y": 0, "origin": origin},
                {"type": "pointerDown", "button": 0},
            ],
        }
    ]
    assert_success(perform_actions(session, actions))

    response = release_actions(session)
    assert assert_success(response) is None

    # Releasing the pressed button must fire pointerup inside the frame.
    assert "pointerup" in session.execute_script("return window.releaseEvents")
