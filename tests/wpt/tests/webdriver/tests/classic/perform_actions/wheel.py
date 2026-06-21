import pytest

from tests.support.sync import Poll
from webdriver.error import MoveTargetOutOfBoundsException, NoSuchWindowException


def test_null_response_value(session, wheel_chain):
    value = wheel_chain.scroll(0, 0, 0, 10).perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_no_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


@pytest.mark.parametrize("origin", ["element", "viewport"])
def test_params_actions_origin_outside_viewport(
    session, test_actions_scroll_page, wheel_chain, origin
):
    if origin == "element":
        origin = session.find.css("#not-scrollable", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        wheel_chain.scroll(-100, -100, 10, 20, origin="viewport").perform()


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
    wheel_chain.scroll(0, 0, 20, 50, origin=target, duration=100).perform()

    scroller = session.find.css("#scroller", all=False)

    def assert_scroll_position(_):
        assert scroller.property("scrollLeft") == pytest.approx(
            20, abs=1.0
        ), "Did not reach scrollLeft position"
        assert scroller.property("scrollTop") == pytest.approx(
            50, abs=1.0
        ), "Did not reach scrollTop position"

    wait = Poll(session, timeout=0.5)
    wait.until(assert_scroll_position)
