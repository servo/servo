import pytest

from webdriver import MoveTargetOutOfBoundsException

from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)


def get_click_coordinates(session):
    return session.execute_script("return window.coords;")


def test_viewport_inside(session, mouse_chain, get_actions_origin_page):
    point = {"x": 50, "y": 50}

    session.url = get_actions_origin_page(
        "width: 100px; height: 50px; background: green;"
    )
    mouse_chain.pointer_move(point["x"], point["y"], origin="viewport").perform()

    click_coords = session.execute_script("return window.coords;")
    assert click_coords["x"] == pytest.approx(point["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(point["y"], abs=1.0)


def test_viewport_outside(session, mouse_chain):
    with pytest.raises(MoveTargetOutOfBoundsException):
        mouse_chain.pointer_move(-50, -50, origin="viewport").perform()


def test_pointer_inside(session, mouse_chain, get_actions_origin_page):
    start_point = {"x": 50, "y": 50}
    offset = {"x": 10, "y": 5}

    session.url = get_actions_origin_page(
        "width: 100px; height: 50px; background: green;"
    )
    mouse_chain.pointer_move(start_point["x"], start_point["y"]).pointer_move(
        offset["x"], offset["y"], origin="pointer"
    ).perform()

    click_coords = session.execute_script("return window.coords;")
    assert click_coords["x"] == pytest.approx(start_point["x"] + offset["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(start_point["y"] + offset["y"], abs=1.0)


def test_pointer_outside(session, mouse_chain):
    with pytest.raises(MoveTargetOutOfBoundsException):
        mouse_chain.pointer_move(-50, -50, origin="pointer").perform()


def test_element_center_point(session, mouse_chain, get_actions_origin_page):
    session.url = get_actions_origin_page(
        "width: 100px; height: 50px; background: green;"
    )
    elem = session.find.css("#inner", all=False)
    center = get_inview_center(elem.rect, get_viewport_rect(session))

    mouse_chain.pointer_move(0, 0, origin=elem).perform()

    click_coords = get_click_coordinates(session)
    assert click_coords["x"] == pytest.approx(center["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(center["y"], abs=1.0)


def test_element_center_point_with_offset(
    session, mouse_chain, get_actions_origin_page
):
    session.url = get_actions_origin_page(
        "width: 100px; height: 50px; background: green;"
    )
    elem = session.find.css("#inner", all=False)
    center = get_inview_center(elem.rect, get_viewport_rect(session))

    mouse_chain.pointer_move(10, 15, origin=elem).perform()

    click_coords = get_click_coordinates(session)
    assert click_coords["x"] == pytest.approx(center["x"] + 10, abs=1.0)
    assert click_coords["y"] == pytest.approx(center["y"] + 15, abs=1.0)


def test_element_in_view_center_point_partly_visible(
    session, mouse_chain, get_actions_origin_page
):
    session.url = get_actions_origin_page(
        """width: 100px; height: 50px; background: green;
                                position: relative; left: -50px; top: -25px;"""
    )
    elem = session.find.css("#inner", all=False)
    center = get_inview_center(elem.rect, get_viewport_rect(session))

    mouse_chain.pointer_move(0, 0, origin=elem).perform()

    click_coords = get_click_coordinates(session)
    assert click_coords["x"] == pytest.approx(center["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(center["y"], abs=1.0)


def test_element_larger_than_viewport(session, mouse_chain, get_actions_origin_page):
    session.url = get_actions_origin_page(
        "width: 300vw; height: 300vh; background: green;"
    )
    elem = session.find.css("#inner", all=False)
    center = get_inview_center(elem.rect, get_viewport_rect(session))

    mouse_chain.pointer_move(0, 0, origin=elem).perform()

    click_coords = get_click_coordinates(session)
    assert click_coords["x"] == pytest.approx(center["x"], abs=1.0)
    assert click_coords["y"] == pytest.approx(center["y"], abs=1.0)


def test_element_outside_of_view_port(session, mouse_chain, get_actions_origin_page):
    session.url = get_actions_origin_page(
        """width: 100px; height: 50px; background: green;
                                position: relative; left: -200px; top: -100px;"""
    )
    elem = session.find.css("#inner", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        mouse_chain.pointer_move(0, 0, origin=elem).perform()
