import pytest

from webdriver.error import MoveTargetOutOfBoundsException, NoSuchWindowException

from . import assert_scroll_position


def test_null_response_value(session, wheel_chain):
    value = wheel_chain.scroll(0, 0, 0, 10).perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_no_browsing_context(session, closed_frame, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


@pytest.mark.parametrize("origin", ["element", "viewport"])
def test_params_actions_origin_outside_viewport(
    session, test_actions_wheel_page, wheel_chain, origin
):
    session.url = test_actions_wheel_page()

    if origin == "element":
        origin = session.find.css("#not-scrollable", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        wheel_chain.scroll(-100, -100, 10, 20, origin=origin).perform()


@pytest.mark.parametrize(
    "delta_x, delta_y",
    [
        (50, 0),
        (0, 60),
        (70, 80),
    ],
    ids=[
        "delta-x",
        "delta-y",
        "delta-x-and-y",
    ],
)
def test_scroll_direction(
    session, test_actions_wheel_page, wheel_chain, delta_x, delta_y
):
    session.url = test_actions_wheel_page()

    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, delta_x, delta_y, origin=target).perform()

    assert_scroll_position(session, target, delta_x, delta_y)


@pytest.mark.parametrize("mode", ["open", "closed"])
def test_scroll_element_in_shadow_tree(
    session, new_tab_classic, test_actions_wheel_page, wheel_chain, mode
):
    session.url = test_actions_wheel_page(shadow=mode)

    shadow_root = session.find.css("#custom-element", all=False).shadow_root
    scrollable = shadow_root.find_element("css selector", "#shadow-scrollable")

    wheel_chain.scroll(0, 0, 55, 75, origin=scrollable).perform()

    assert_scroll_position(session, scrollable, 55, 75)


@pytest.mark.parametrize("scale", ["0.5", "1.0", "1.5"])
def test_scroll_position_for_scaled_layout_viewport(
    session, new_tab_classic, inline, wheel_chain, scale
):
    session.url = inline(f"""
        <meta name="viewport" content="width=device-width,initial-scale={scale}">
        <div id="scroller" style="overflow: auto; width: 250px; height: 150px">
          <iframe srcdoc="foo" style="width: 200px; height: 100px"></iframe>
          <div style="height: 2000px; width: 2000px"></div>
        </div>
    """)

    target = session.find.css("iframe", all=False)
    wheel_chain.scroll(0, 0, 60, 80, origin=target, duration=100).perform()

    scroller = session.find.css("#scroller", all=False)
    assert_scroll_position(session, scroller, 60, 80)
