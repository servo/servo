import pytest

from tests.support.asserts import assert_success
from tests.support.image import png_dimensions
from tests.support.inline import iframe, inline

from . import element_rect


DEFAULT_CSS_STYLE = """
    <style>
      div, iframe {
        display: block;
        border: 1px solid blue;
        width: 10em;
        height: 10em;
      }
    </style>
"""

DEFAULT_CONTENT = "<div>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</div>"


def take_element_screenshot(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/screenshot".format(
            session_id=session.session_id,
            element_id=element_id,
        )
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_source_origin(session, url, domain):
    session.url = inline("""{0}{1}""".format(DEFAULT_CSS_STYLE, DEFAULT_CONTENT))
    element = session.find.css("div", all=False)
    rect = element_rect(session, element)

    response = take_element_screenshot(session, element.id)
    reference_screenshot = assert_success(response)
    assert png_dimensions(reference_screenshot) == (rect["width"], rect["height"])

    iframe_content = "<style>body {{ margin: 0; }}</style>{}".format(DEFAULT_CONTENT)
    session.url = inline("""{0}{1}""".format(
        DEFAULT_CSS_STYLE, iframe(iframe_content, domain=domain)))
    frame_element = session.find.css("iframe", all=False)
    frame_rect = element_rect(session, frame_element)

    response = take_element_screenshot(session, frame_element.id)
    screenshot = assert_success(response)
    assert png_dimensions(screenshot) == (frame_rect["width"], frame_rect["height"])

    assert screenshot == reference_screenshot
