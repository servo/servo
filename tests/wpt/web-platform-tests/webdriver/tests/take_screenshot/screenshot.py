from tests.support.asserts import assert_error, assert_png, assert_success
from tests.support.image import png_dimensions
from tests.support.inline import inline

from . import viewport_dimensions


def take_screenshot(session):
    return session.transport.send(
        "GET", "session/{session_id}/screenshot".format(**vars(session)))


def test_no_browsing_context(session, closed_window):
    response = take_screenshot(session)
    assert_error(response, "no such window")


def test_format_and_dimensions(session):
    session.url = inline("<input>")

    response = take_screenshot(session)
    value = assert_success(response)

    assert_png(value)
    assert png_dimensions(value) == viewport_dimensions(session)
