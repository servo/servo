import math
import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def center_point(element):
    """Calculates the in-view center point of a web element."""
    inner_width, inner_height = element.session.execute_script(
        "return [window.innerWidth, window.innerHeight]")
    rect = element.rect

    # calculate the intersection of the rect that is inside the viewport
    visible = {
        "left": max(0, min(rect["x"], rect["x"] + rect["width"])),
        "right": min(inner_width, max(rect["x"], rect["x"] + rect["width"])),
        "top": max(0, min(rect["y"], rect["y"] + rect["height"])),
        "bottom": min(inner_height, max(rect["y"], rect["y"] + rect["height"])),
    }

    # arrive at the centre point of the visible rectangle
    x = (visible["left"] + visible["right"]) / 2.0
    y = (visible["top"] + visible["bottom"]) / 2.0

    # convert to CSS pixels, as centre point can be float
    return (math.floor(x), math.floor(y))


def square(size):
    return inline("""
        <style>
        body {{ margin: 0 }}

        div {{
          background: blue;
          width: {size}px;
          height: {size}px;
        }}
        </style>

        <div id=target></div>

        <script>
        window.clicks = [];
        let div = document.querySelector("div");
        div.addEventListener("click", function(e) {{ window.clicks.push([e.clientX, e.clientY]) }});
        </script>
        """.format(size=size))


def assert_one_click(session):
    """Asserts there has only been one click, and returns that."""
    clicks = session.execute_script("return window.clicks")
    assert len(clicks) == 1
    return tuple(clicks[0])


def test_entirely_in_view(session):
    session.url = square(444)
    element = session.find.css("#target", all=False)

    response = element_click(session, element)
    assert_success(response)

    click_point = assert_one_click(session)
    assert click_point == (222, 222)


@pytest.mark.parametrize("size", range(1, 11))
def test_css_pixel_rounding(session, size):
    session.url = square(size)
    element = session.find.css("#target", all=False)
    expected_click_point = center_point(element)

    response = element_click(session, element)
    assert_success(response)

    actual_click_point = assert_one_click(session)
    assert actual_click_point == expected_click_point
