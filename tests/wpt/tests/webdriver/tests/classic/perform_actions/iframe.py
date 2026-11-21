import pytest

from webdriver import WebElement
from tests.support.classic.asserts import assert_success
from tests.support.classic.frame_coordinates import (
    assert_clicked_at,
    containing_document,
    in_view_center_point,
    scroll_to,
    target_document,
)


def perform_actions(session, actions):
    return session.transport.send(
        "POST",
        "/session/{session_id}/actions".format(session_id=session.session_id),
        {"actions": actions})


def release_actions(session):
    return session.transport.send(
        "DELETE",
        "/session/{session_id}/actions".format(session_id=session.session_id))


def click_element_actions(element, x=0, y=0):
    # session.transport.send posts the action dicts as raw JSON, so the element has to be
    # referenced in wire format rather than passed as a WebElement.
    origin = {WebElement.identifier: element.id}
    return [
        {
            "type": "pointer",
            "id": "pointer_id",
            "parameters": {"pointerType": "mouse"},
            "actions": [
                {"type": "pointerMove", "x": x, "y": y, "origin": origin},
                {"type": "pointerDown", "button": 0},
                {"type": "pointerUp", "button": 0},
            ],
        }
    ]


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_perform_actions_on_element_in_frame(session, inline, iframe, domain):
    clicker = "<button id='b' onclick=\"document.title='clicked'\">go</button>"
    session.url = inline(iframe(clicker, domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    button = session.find.css("#b", all=False)

    try:
        response = perform_actions(session, click_element_actions(button))
        assert assert_success(response) is None

        # The click synthesized by the actions must fire inside the frame.
        assert session.execute_script("return document.title") == "clicked"
    finally:
        # Reset the input state so a lingering pointer press cannot leak into later tests.
        release_actions(session)


# An element-relative pointer origin resolves to the element's in-view center point, which has to
# be translated out of the frame's coordinate space and into the main frame's before the
# synthesized pointer events can be dispatched. The tests below record events on the subframe's
# window rather than on the target, so an error in that translation stays observable as a
# coordinate offset instead of collapsing into "nothing happened".


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("main_scroll, frame_scroll", [
    (0, 0),
    (100, 0),
    (0, 500),
    (100, 500),
], ids=["unscrolled", "scrolled_main_frame", "scrolled_frame", "both_scrolled"])
def test_perform_actions_on_element_in_scrolled_frame(session, inline, iframe, domain, main_scroll, frame_scroll):
    session.url = inline(containing_document(
        iframe(target_document(frame_scroll), domain=domain),
        left=100, top=150, width=300, height=200, body_height=2000))

    if main_scroll:
        scroll_to(session, main_scroll)

    session.switch_to_frame(session.find.css("iframe", all=False))

    if frame_scroll:
        scroll_to(session, frame_scroll)

    expected = in_view_center_point(session)
    target = session.find.css("#target", all=False)

    try:
        response = perform_actions(session, click_element_actions(target))
        assert assert_success(response) is None

        assert_clicked_at(session, expected, frame_scroll)
    finally:
        release_actions(session)


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_perform_actions_on_element_in_transformed_frame(session, inline, iframe, domain):
    # A CSS transform on the frame owner has to be applied on the way out to the main frame, or
    # the pointer lands 30x40 px away from the target.
    session.url = inline(containing_document(
        iframe(target_document(0), domain=domain),
        left=100, top=150, width=300, height=200, body_height=800,
        frame_style="transform: translate(30px, 40px); transform-origin: 0 0;"))

    session.switch_to_frame(session.find.css("iframe", all=False))

    expected = in_view_center_point(session)
    target = session.find.css("#target", all=False)

    try:
        response = perform_actions(session, click_element_actions(target))
        assert assert_success(response) is None

        assert_clicked_at(session, expected)
    finally:
        release_actions(session)


def test_perform_actions_on_element_in_nested_cross_origin_frame(session, inline, iframe):
    # Nest the top document's own origin inside a cross-origin frame, so the conversion out to the
    # main frame has to traverse two levels rather than one.
    innermost = iframe(target_document(0), domain="")
    middle = containing_document(
        innermost, left=50, top=40, width=300, height=200, body_height=300)
    session.url = inline(containing_document(
        iframe(middle, domain="alt"),
        left=60, top=80, width=400, height=300, body_height=1200))

    session.switch_to_frame(session.find.css("iframe", all=False))
    session.switch_to_frame(session.find.css("iframe", all=False))

    expected = in_view_center_point(session)
    target = session.find.css("#target", all=False)

    try:
        response = perform_actions(session, click_element_actions(target))
        assert assert_success(response) is None

        assert_clicked_at(session, expected)
    finally:
        release_actions(session)
