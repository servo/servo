import pytest

from tests.actions.support.mouse import assert_move_to_coordinates, get_center
from tests.actions.support.refine import get_events, filter_dict


_DBLCLICK_INTERVAL = 640


# Using local fixtures because we want to start a new session between
# each test, otherwise the clicks in each test interfere with each other.
@pytest.fixture(autouse=True)
def release_actions(dblclick_session, request):
    # release all actions after each test
    # equivalent to a teardown_function, but with access to session fixture
    request.addfinalizer(dblclick_session.actions.release)


@pytest.fixture
def dblclick_session(new_session, url, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    session.url = url("/webdriver/tests/actions/support/test_actions_wdspec.html")

    return session


@pytest.fixture
def mouse_chain(dblclick_session):
    return dblclick_session.actions.sequence(
        "pointer",
        "pointer_id",
        {"pointerType": "mouse"})


@pytest.mark.parametrize("click_pause", [0, 200])
def test_dblclick_at_coordinates(dblclick_session, mouse_chain, click_pause):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"]) \
        .click() \
        .pause(click_pause) \
        .click() \
        .perform()
    events = get_events(dblclick_session)
    assert_move_to_coordinates(div_point, "outer", events)
    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "dblclick", "button": 0},
    ]
    assert len(events) == 8
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


def test_dblclick_with_pause_after_second_pointerdown(dblclick_session, mouse_chain):
        outer = dblclick_session.find.css("#outer", all=False)
        center = get_center(outer.rect)
        mouse_chain \
            .pointer_move(int(center["x"]), int(center["y"])) \
            .click() \
            .pointer_down() \
            .pause(_DBLCLICK_INTERVAL + 10) \
            .pointer_up() \
            .perform()
        events = get_events(dblclick_session)
        expected = [
            {"type": "mousedown", "button": 0},
            {"type": "mouseup", "button": 0},
            {"type": "click", "button": 0},
            {"type": "mousedown", "button": 0},
            {"type": "mouseup", "button": 0},
            {"type": "click", "button": 0},
            {"type": "dblclick", "button": 0},
        ]
        assert len(events) == 8
        filtered_events = [filter_dict(e, expected[0]) for e in events]
        assert expected == filtered_events[1:]


def test_no_dblclick(dblclick_session, mouse_chain):
        outer = dblclick_session.find.css("#outer", all=False)
        center = get_center(outer.rect)
        mouse_chain \
            .pointer_move(int(center["x"]), int(center["y"])) \
            .click() \
            .pause(_DBLCLICK_INTERVAL + 10) \
            .click() \
            .perform()
        events = get_events(dblclick_session)
        expected = [
            {"type": "mousedown", "button": 0},
            {"type": "mouseup", "button": 0},
            {"type": "click", "button": 0},
            {"type": "mousedown", "button": 0},
            {"type": "mouseup", "button": 0},
            {"type": "click", "button": 0},
        ]
        assert len(events) == 7
        filtered_events = [filter_dict(e, expected[0]) for e in events]
        assert expected == filtered_events[1:]
