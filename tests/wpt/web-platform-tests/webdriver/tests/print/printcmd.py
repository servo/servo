from base64 import decodebytes

import pytest

from tests.support.asserts import assert_error, assert_success


def do_print(session, options):
    return session.transport.send(
        "POST", "session/{session_id}/print".format(**vars(session)),
        options)


def assert_pdf(data):
    assert data.startswith(b"%PDF-"), "Decoded data starts with the PDF signature"
    assert data.endswith(b"%%EOF\n"), "Decoded data ends with the EOF flag"


def test_no_top_browsing_context(session, closed_window):
    response = do_print(session, {})
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = do_print(session, {})
    value = assert_success(response)
    pdf = decodebytes(value.encode())
    assert_pdf(pdf)


def test_html_document(session, inline):
    session.url = inline("Test")

    response = do_print(session, {
        "page": {"width": 10,
                 "height": 20},
        "shrinkToFit": False
    })
    value = assert_success(response)
    pdf = decodebytes(value.encode())
    # TODO: Test that the output is reasonable
    assert_pdf(pdf)

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
    assert_pdf(pdf)


@pytest.mark.parametrize("options", [{"orientation": 0},
                                     {"orientation": "foo"},
                                     {"scale": "1"},
                                     {"scale": 3},
                                     {"scale": 0.01},
                                     {"margin": {"top": "1"}},
                                     {"margin": {"bottom": -1}},
                                     {"page": {"height": False}},
                                     {"shrinkToFit": "false"}])
def test_invalid(session, options):
    response = do_print(session, options)
    assert_error(response, "invalid argument")
