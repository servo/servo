import pytest

from tests.support.asserts import assert_success
from tests.support.image import png_dimensions
from tests.support.inline import iframe, inline

from . import viewport_dimensions


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

DEFAULT_CONTENT = "<div>Lorem ipsum dolor sit amet.</div>"


def take_screenshot(session):
    return session.transport.send(
        "GET", "session/{session_id}/screenshot".format(**vars(session)))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_source_origin(session, url, domain):
    session.url = inline("""{0}{1}""".format(DEFAULT_CSS_STYLE, DEFAULT_CONTENT))

    response = take_screenshot(session)
    reference_screenshot = assert_success(response)
    assert png_dimensions(reference_screenshot) == viewport_dimensions(session)

    iframe_content = "<style>body {{ margin: 0; }}</style>{}".format(DEFAULT_CONTENT)
    session.url = inline("""{0}{1}""".format(
        DEFAULT_CSS_STYLE, iframe(iframe_content, domain=domain)))

    response = take_screenshot(session)
    screenshot = assert_success(response)
    assert png_dimensions(screenshot) == viewport_dimensions(session)

    assert screenshot == reference_screenshot
