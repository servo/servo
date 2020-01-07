import pytest

from tests.support.asserts import assert_success
from tests.support.image import png_dimensions
from tests.support.inline import iframe, inline

from . import element_dimensions, viewport_dimensions

DEFAULT_CONTENT = "<div id='content'>Lorem ipsum dolor sit amet.</div>"

REFERENCE_CONTENT = "<div id='outer'>{}</div>".format(DEFAULT_CONTENT)
REFERENCE_STYLE = """
    <style>
      #outer {
        display: block;
        margin: 0;
        border: 0;
        width: 200px;
        height: 200px;
      }
      #content {
        display: block;
        margin: 0;
        border: 0;
        width: 100px;
        height: 100px;
        background: green;
      }
    </style>
"""

OUTER_IFRAME_STYLE = """
    <style>
      iframe {
        display: block;
        margin: 0;
        border: 0;
        width: 200px;
        height: 200px;
      }
    </style>
"""

INNER_IFRAME_STYLE = """
    <style>
      body {
        margin: 0;
      }
      div {
        display: block;
        margin: 0;
        border: 0;
        width: 100px;
        height: 100px;
        background: green;
      }
    </style>
"""


def take_screenshot(session):
    return session.transport.send(
        "GET", "session/{session_id}/screenshot".format(**vars(session)))


def test_frame_content(session):
    # Create a reference element which looks exactly like the frame's content
    session.url = inline("{0}{1}".format(REFERENCE_STYLE, REFERENCE_CONTENT))

    # Capture the inner content as reference image
    ref_el = session.find.css("div", all=False)
    ref_screenshot = ref_el.screenshot()
    ref_dimensions = element_dimensions(session, ref_el)

    assert ref_dimensions != viewport_dimensions
    assert png_dimensions(ref_screenshot) == ref_dimensions

    # Capture the frame's content
    iframe_content = "{0}{1}".format(INNER_IFRAME_STYLE, DEFAULT_CONTENT)
    session.url = inline("""{0}{1}""".format(OUTER_IFRAME_STYLE, iframe(iframe_content)))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    frame_dimensions = viewport_dimensions(session)
    assert frame_dimensions == ref_dimensions

    response = take_screenshot(session)
    frame_screenshot = assert_success(response)

    assert png_dimensions(frame_screenshot) == ref_dimensions
    assert frame_screenshot == ref_screenshot


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_source_origin(session, domain):
    session.url = inline("{0}{1}".format(REFERENCE_STYLE, REFERENCE_CONTENT))

    response = take_screenshot(session)
    reference_screenshot = assert_success(response)
    assert png_dimensions(reference_screenshot) == viewport_dimensions(session)

    iframe_content = "{0}{1}".format(INNER_IFRAME_STYLE, DEFAULT_CONTENT)
    session.url = inline("""{0}{1}""".format(
        OUTER_IFRAME_STYLE, iframe(iframe_content, domain=domain)))

    response = take_screenshot(session)
    screenshot = assert_success(response)
    assert png_dimensions(screenshot) == viewport_dimensions(session)

    assert screenshot == reference_screenshot
