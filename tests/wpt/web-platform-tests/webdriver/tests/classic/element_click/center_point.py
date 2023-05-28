import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import center_point


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def square(inline, size):
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


def test_entirely_in_view(session, inline):
    session.url = square(inline, 300)
    element = session.find.css("#target", all=False)

    response = element_click(session, element)
    assert_success(response)

    click_point = assert_one_click(session)
    assert click_point == (150, 150)


@pytest.mark.parametrize("size", range(1, 11))
def test_css_pixel_rounding(session, inline, size):
    session.url = square(inline, size)
    element = session.find.css("#target", all=False)
    expected_click_point = center_point(element)

    response = element_click(session, element)
    assert_success(response)

    actual_click_point = assert_one_click(session)
    assert actual_click_point == expected_click_point
