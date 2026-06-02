import pytest

from tests.support.asserts import assert_error, assert_png, assert_success
from tests.support.image import png_dimensions

from . import viewport_dimensions


def take_screenshot(session):
    return session.transport.send(
        "GET", "session/{session_id}/screenshot".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = take_screenshot(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame, inline):
    session.url = inline("<input>")

    response = take_screenshot(session)
    value = assert_success(response)

    assert_png(value)
    assert png_dimensions(value) == viewport_dimensions(session)


def test_format_and_dimensions(session, inline):
    session.url = inline("<input>")

    response = take_screenshot(session)
    value = assert_success(response)

    assert_png(value)
    assert png_dimensions(value) == viewport_dimensions(session)


def test_huge_document_clips_to_viewport(session, inline):
    width = "32768px"
    height = "32768px"

    session.url = inline(f"<div style='width: {width}; height: {height}; background-color: black;'></div>")

    response = take_screenshot(session)
    value = assert_success(response)

    assert_png(value)
    assert png_dimensions(value) == viewport_dimensions(session)
