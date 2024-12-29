import pytest
from webdriver.error import (
    InvalidArgumentException,
    MoveTargetOutOfBoundsException,
    NoSuchWindowException,
    StaleElementReferenceException,
)

from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)
from tests.classic.perform_actions.support.refine import get_events
from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import center_point, filter_dict
from tests.support.sync import Poll

from . import assert_pointer_events, record_pointer_events


def test_null_response_value(session, mouse_chain):
    value = mouse_chain.click().perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, mouse_chain):
    with pytest.raises(NoSuchWindowException):
        mouse_chain.click().perform()


def test_no_browsing_context(session, closed_frame, mouse_chain):
    with pytest.raises(NoSuchWindowException):
        mouse_chain.click().perform()


def test_pointer_down_closes_browsing_context(
    session, configuration, http_new_tab, inline, mouse_chain
):
    session.url = inline(
        """<input onpointerdown="window.close()">close</input>""")
    origin = session.find.css("input", all=False)

    with pytest.raises(NoSuchWindowException):
        mouse_chain.pointer_move(0, 0, origin=origin) \
            .pointer_down(button=0) \
            .pause(100 * configuration["timeout_multiplier"]) \
            .pointer_up(button=0) \
            .perform()


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, mouse_chain, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    with pytest.raises(StaleElementReferenceException):
        mouse_chain.click(element=element).perform()


@pytest.mark.parametrize("origin", ["element", "pointer", "viewport"])
def test_params_actions_origin_outside_viewport(session, test_actions_page, mouse_chain, origin):
    if origin == "element":
        origin = session.find.css("#outer", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        mouse_chain.pointer_move(-100, -100, origin=origin).perform()


def test_click_at_coordinates(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"], duration=1000) \
        .click() \
        .perform()
    events = get_events(session)
    assert len(events) == 4
    assert_move_to_coordinates(div_point, "outer", events)
    for e in events:
        if e["type"] != "mousedown":
            assert e["buttons"] == 0
        assert e["button"] == 0
    expected = [
        {"type": "mousedown", "buttons": 1},
        {"type": "mouseup", "buttons": 0},
        {"type": "click", "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


def test_context_menu_at_coordinates(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"]) \
        .pointer_down(button=2) \
        .pointer_up(button=2) \
        .perform()

    events = get_events(session)
    assert len(events) == 4

    expected = [
        {"type": "mousedown", "button": 2, "buttons": 2},
        {"type": "contextmenu", "button": 2, "buttons": 2},
    ]
    # Some browsers in some platforms may dispatch `contextmenu` event as a
    # a default action of `mouseup`.  In the case, `.buttons` of the event
    # should be 0.
    anotherExpected = [
        {"type": "mousedown", "button": 2, "buttons": 2},
        {"type": "contextmenu", "button": 2, "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    mousedown_contextmenu_events = [
        x for x in filtered_events
        if x["type"] in ["mousedown", "contextmenu"]
    ]
    assert mousedown_contextmenu_events in [expected, anotherExpected]


def test_middle_click(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"]) \
        .pointer_down(button=1) \
        .pointer_up(button=1) \
        .perform()

    events = get_events(session)
    assert len(events) == 3

    expected = [
        {"type": "mousedown", "button": 1, "buttons": 4},
        {"type": "mouseup", "button": 1, "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    mousedown_mouseup_events = [
        x for x in filtered_events
        if x["type"] in ["mousedown", "mouseup"]
    ]
    assert expected == mousedown_mouseup_events


def test_click_element_center(session, test_actions_page, mouse_chain):
    outer = session.find.css("#outer", all=False)
    center = get_inview_center(outer.rect, get_viewport_rect(session))
    mouse_chain.click(element=outer).perform()
    events = get_events(session)
    assert len(events) == 4
    event_types = [e["type"] for e in events]
    assert ["mousemove", "mousedown", "mouseup", "click"] == event_types
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == pytest.approx(center["x"], abs=1.0)
            assert e["pageY"] == pytest.approx(center["y"], abs=1.0)
            assert e["target"] == "outer"


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_click_element_in_shadow_tree(
    session, get_test_page, mouse_chain, mode, nested
):
    session.url = get_test_page(
        shadow_doc="""
        <div id="pointer-target"
             style="width: 10px; height: 10px; background-color:blue;">
        </div>""",
        shadow_root_mode=mode,
        nested_shadow_dom=nested,
    )

    shadow_root = session.find.css("custom-element", all=False).shadow_root
    if nested:
        shadow_root = shadow_root.find_element("css selector", "inner-custom-element").shadow_root

    target = shadow_root.find_element("css selector", "#pointer-target")
    record_pointer_events(session, target)

    mouse_chain.click(element=target).perform()
    assert_pointer_events(
        session,
        expected_events=["pointerdown", "pointerup"],
        target="pointer-target",
        pointer_type="mouse",
    )


def test_click_navigation(session, url, inline):
    destination = url("/webdriver/tests/support/html/test_actions.html")
    start = inline("<a href=\"{}\" id=\"link\">destination</a>".format(destination))

    def click(link):
        mouse_chain = session.actions.sequence(
            "pointer", "pointer_id", {"pointerType": "mouse"})
        mouse_chain.click(element=link).perform()

    session.url = start
    error_message = "Did not navigate to %s" % destination

    click(session.find.css("#link", all=False))
    Poll(session, message=error_message).until(lambda s: s.url == destination)
    # repeat steps to check behaviour after document unload
    session.url = start
    click(session.find.css("#link", all=False))
    Poll(session, message=error_message).until(lambda s: s.url == destination)


@pytest.mark.parametrize("x, y, event_count", [
    (0, 0, 0),
    (1, 0, 1),
    (0, 1, 1),
], ids=["default value", "x", "y"])
def test_move_to_position_in_viewport(
    session, test_actions_page, mouse_chain, x, y, event_count
):
    mouse_chain.pointer_move(x, y).perform()
    events = get_events(session)
    assert len(events) == event_count

    # Move again to check that no further mouse move event is emitted.
    mouse_chain.pointer_move(x, y).perform()
    events = get_events(session)
    assert len(events) == event_count


@pytest.mark.parametrize("origin", ["viewport", "pointer", "element"])
def test_move_to_origin_position_within_frame(
    session, iframe, inline, mouse_chain, origin
):
    session.url = inline(
        iframe(
            """
        <input>
        <script>
            "use strict;"

            var allEvents = { events: [] };
            window.addEventListener("mousemove", e => {
                allEvents.events.push([
                    e.clientX,
                    e.clientY,
                ]);
            });
        </script>
    """
        )
    )

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    elem = session.find.css("input", all=False)
    elem_center_point = center_point(elem)

    offset = [10, 5]

    if origin == "element":
        origin = elem
        target_point = [
            elem_center_point[0] + offset[0],
            elem_center_point[1] + offset[1],
        ]
    else:
        target_point = offset

    mouse_chain.pointer_move(offset[0], offset[1], origin=origin).perform()

    events = get_events(session)
    assert len(events) == 1
    assert events[0] == target_point


@pytest.mark.parametrize("missing", ["x", "y"])
def test_missing_coordinates(session, test_actions_page, mouse_chain, missing):
    outer = session.find.css("#outer", all=False)
    actions = mouse_chain.pointer_move(x=0, y=0, origin=outer)
    del actions._actions[-1][missing]
    with pytest.raises(InvalidArgumentException):
        actions.perform()


def test_invalid_element_origin(session, test_actions_page, mouse_chain):
    outer = session.find.css("#outer", all=False)
    actions = mouse_chain.pointer_move(
        x=0, y=0, origin={"type": "element", "element": {"sharedId": outer.id}}
    )
    with pytest.raises(InvalidArgumentException):
        actions.perform()
