# META: timeout=long
from base64 import decodebytes

import pytest

from tests.support.asserts import assert_error, assert_pdf, assert_success

from . import do_print


def test_no_top_browsing_context(session, closed_window):
    response = do_print(session, {})
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = do_print(session, {})
    value = assert_success(response)
    assert_pdf(value)


def test_html_document(session, inline):
    session.url = inline("Test")

    response = do_print(session, {
        "page": {"width": 10,
                 "height": 20},
        "shrinkToFit": False
    })
    value = assert_success(response)
    # TODO: Test that the output is reasonable
    assert_pdf(value)


def test_large_html_document(session, inline):
    session.url = inline("<canvas id=\"image\"></canvas>")

    session.execute_script(
        """
        const width = 700;
        const height = 900;

        const canvas = document.getElementById("image");
        const context = canvas.getContext("2d");

        canvas.width = width;
        canvas.height = height;

        for (let x = 0; x < width; ++x) {
            for (let y = 0; y < height; ++y) {
                const colourHex = Math.floor(Math.random() * 0xffffff).toString(16);

                context.fillStyle = `#${colourHex}`;
                context.fillRect(x, y, 1, 1);
            }
        }
        """
    )

    response = do_print(session, {})
    value = assert_success(response)
    pdf = decodebytes(value.encode())

    # This was added to test the fix for a bug in firefox where a PDF larger
    # than 500kb would cause an error. If the resulting PDF is smaller than that
    # it could pass incorrectly.
    assert len(pdf) > 500000
    assert_pdf(value)


@pytest.mark.parametrize("ranges,expected", [
    (["2-4"], ["Page 2", "Page 3", "Page 4"]),
    (["2-4", "2-3"], ["Page 2", "Page 3", "Page 4"]),
    (["2-4", "3-5"], ["Page 2", "Page 3", "Page 4", "Page 5"]),
    (["9-"], ["Page 9", "Page 10"]),
    (["-2"], ["Page 1", "Page 2"]),
    (["7"], ["Page 7"]),
    ([7],["Page 7"]),
    (["-2", "9-", "7"], ["Page 1", "Page 2", "Page 7", "Page 9", "Page 10"]),
    (["-2", "9-", 7], ["Page 1", "Page 2", "Page 7", "Page 9", "Page 10"]),
    (["-5", "2-"], ["Page 1", "Page 2", "Page 3", "Page 4", "Page 5", "Page 6", "Page 7", "Page 8", "Page 9", "Page 10"]),
    ([], ["Page 1", "Page 2", "Page 3", "Page 4", "Page 5", "Page 6", "Page 7", "Page 8", "Page 9", "Page 10"]),
])
def test_page_ranges_document(session, inline, load_pdf_http, ranges, expected):
    session.url = inline("""
<style>
div {page-break-after: always}
</style>

<div>Page 1</div>
<div>Page 2</div>
<div>Page 3</div>
<div>Page 4</div>
<div>Page 5</div>
<div>Page 6</div>
<div>Page 7</div>
<div>Page 8</div>
<div>Page 9</div>
<div>Page 10</div>""")

    response = do_print(session, {
        "pageRanges": ranges
    })
    value = assert_success(response)
    # TODO: Test that the output is reasonable
    assert_pdf(value)

    load_pdf_http(value)
    pages = session.execute_async_script("""let callback = arguments[arguments.length - 1];
window.getText().then(pages => callback(pages));""")
    assert pages == expected


@pytest.mark.parametrize("options", [{"orientation": 0},
                                     {"orientation": "foo"},
                                     {"scale": "1"},
                                     {"scale": 3},
                                     {"scale": 0.01},
                                     {"margin": {"top": "1"}},
                                     {"margin": {"bottom": -1}},
                                     {"page": {"height": False}},
                                     {"shrinkToFit": "false"},
                                     {"pageRanges": ["3-2"]},
                                     {"pageRanges": ["a-2"]},
                                     {"pageRanges": ["1:2"]},
                                     {"pageRanges": ["1-2-3"]},
                                     {"pageRanges": [None]},
                                     {"pageRanges": ["1-2", {}]}])
def test_page_ranges_invalid(session, options):
    response = do_print(session, options)
    assert_error(response, "invalid argument")
